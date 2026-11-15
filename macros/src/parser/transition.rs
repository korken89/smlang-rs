use super::event::Event;
use super::input_state::InputState;
use super::output_state::OutputState;
use super::AsyncIdent;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::fmt;
use syn::{bracketed, parse, token, Ident, Token};

const EVENTLESS_WILDCARD: &str =
    "Eventless transitions cannot use a wildcard input state; add an event (`_ + Event`)";
const EVENTLESS_NO_TARGET: &str =
    "expected `+ Event`, or an explicit target state (`= State`) for an eventless transition";

#[derive(Debug, Clone)]
pub struct StateTransition {
    pub in_state: InputState,
    /// `None` for an eventless transition.
    pub event: Option<Event>,
    pub guard: Option<GuardExpression>,
    pub action: Option<AsyncIdent>,
    pub out_state: OutputState,
}

#[derive(Debug)]
pub struct StateTransitions {
    pub in_states: Vec<InputState>,
    /// `None` for an eventless transition.
    pub event: Option<Event>,
    pub guard: Option<GuardExpression>,
    pub action: Option<AsyncIdent>,
    pub out_state: OutputState,
}

impl parse::Parse for StateTransitions {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        // parse the input pattern
        let mut in_states = Vec::new();
        loop {
            let in_state: InputState = input.parse()?;
            in_states.push(in_state);
            if input.parse::<Token![|]>().is_err() {
                break;
            };
        }

        // Make sure that if a wildcard is used, it is the only input state
        if in_states.len() > 1 {
            for in_state in &in_states {
                if in_state.wildcard {
                    return Err(parse::Error::new(
                        in_state.ident.span(),
                        "Wildcards already include all states, so should not be used with input state patterns.",
                    ));
                }
            }
        }
        // Event. It is optional: a transition without `+ Event` is eventless.
        let event: Option<Event> = if input.peek(Token![+]) {
            Some(input.parse()?)
        } else {
            None
        };

        // Eventless transitions cannot start from a wildcard: every state would become
        // transient, including the target.
        if event.is_none() {
            if let Some(wildcard) = in_states.iter().find(|in_state| in_state.wildcard) {
                return Err(parse::Error::new(wildcard.ident.span(), EVENTLESS_WILDCARD));
            }
        }

        // Possible guard
        let guard = if input.peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            Some(GuardExpression::parse(&content)?)
        } else {
            None
        };

        // Possible action
        let action = if input.parse::<Token![/]>().is_ok() {
            let is_async = input.parse::<token::Async>().is_ok();
            let action: Ident = input.parse()?;
            Some(AsyncIdent {
                ident: action,
                is_async,
            })
        } else {
            None
        };

        let has_target_marker = input.peek(Token![=]);
        let out_state: OutputState = input.parse()?;

        // Eventless transitions need an explicit target state. The target is either omitted
        // (`S / a`, `S [g]`, `S`, which usually means that the event was forgotten) or
        // written as the internal transition `= _`.
        if event.is_none() && out_state.internal_transition {
            let span = if has_target_marker {
                out_state.ident.span()
            } else {
                in_states
                    .last()
                    .map_or_else(Span::call_site, |in_state| in_state.ident.span())
            };
            return Err(parse::Error::new(span, EVENTLESS_NO_TARGET));
        }

        Ok(Self {
            in_states,
            event,
            guard,
            action,
            out_state,
        })
    }
}
#[derive(Debug, Clone)]
pub enum GuardExpression {
    Guard(AsyncIdent),
    Not(Box<GuardExpression>),
    Group(Box<GuardExpression>),
    And(Box<GuardExpression>, Box<GuardExpression>),
    Or(Box<GuardExpression>, Box<GuardExpression>),
}
impl fmt::Display for GuardExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuardExpression::Guard(async_ident) => write!(f, "{}", async_ident),
            GuardExpression::Not(expr) => write!(f, "!{}", expr),
            GuardExpression::Group(expr) => write!(f, "({})", expr),
            GuardExpression::And(lhs, rhs) => {
                write!(f, "{} && {}", lhs, rhs)
            }
            GuardExpression::Or(lhs, rhs) => {
                write!(f, "{} || {}", lhs, rhs)
            }
        }
    }
}
impl GuardExpression {
    pub fn to_token_stream<F>(&self, visit: &mut F) -> TokenStream
    where
        F: FnMut(&AsyncIdent) -> TokenStream,
    {
        match self {
            GuardExpression::Guard(async_ident) => async_ident.to_token_stream(visit),
            GuardExpression::Not(expr) => {
                let expr_tokens = expr.to_token_stream(visit);
                quote! { !#expr_tokens }
            }
            GuardExpression::Group(expr) => {
                let expr_tokens = expr.to_token_stream(visit);
                quote! { (#expr_tokens) }
            }
            GuardExpression::And(lhs, rhs) => {
                let lhs_tokens = lhs.to_token_stream(visit);
                let rhs_tokens = rhs.to_token_stream(visit);
                quote! { #lhs_tokens && #rhs_tokens }
            }
            GuardExpression::Or(lhs, rhs) => {
                let lhs_tokens = lhs.to_token_stream(visit);
                let rhs_tokens = rhs.to_token_stream(visit);
                quote! { #lhs_tokens || #rhs_tokens }
            }
        }
    }
}

pub fn visit_guards<F>(expr: &GuardExpression, mut visit_guard: F) -> Result<(), parse::Error>
where
    F: FnMut(&AsyncIdent) -> Result<(), parse::Error>,
{
    let mut stack = vec![expr];
    while let Some(node) = stack.pop() {
        match node {
            GuardExpression::Guard(guard) => {
                visit_guard(guard)?;
            }
            GuardExpression::Not(inner) | GuardExpression::Group(inner) => {
                stack.push(inner.as_ref());
            }
            GuardExpression::And(left, right) | GuardExpression::Or(left, right) => {
                stack.push(left.as_ref());
                stack.push(right.as_ref());
            }
        }
    }
    Ok(())
}

impl parse::Parse for GuardExpression {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        parse_or(input)
    }
}

fn parse_or(input: parse::ParseStream) -> syn::Result<GuardExpression> {
    let mut left = parse_and(input)?;
    while input.peek(Token![||]) {
        let _or: Token![||] = input.parse()?;
        let right = parse_and(input)?;
        left = GuardExpression::Or(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_and(input: parse::ParseStream) -> syn::Result<GuardExpression> {
    let mut left = parse_not(input)?;
    while input.peek(Token![&&]) {
        let _and: Token![&&] = input.parse()?;
        let right = parse_not(input)?;
        left = GuardExpression::And(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_not(input: parse::ParseStream) -> syn::Result<GuardExpression> {
    if input.peek(Token![!]) {
        let _not: Token![!] = input.parse()?;
        let expr = parse_primary(input)?;
        return Ok(GuardExpression::Not(Box::new(expr)));
    }
    parse_primary(input)
}

fn parse_primary(input: parse::ParseStream) -> syn::Result<GuardExpression> {
    if input.peek(token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let expr = parse_or(&content)?;
        return Ok(GuardExpression::Group(Box::new(expr)));
    }

    if input.peek(Token![async]) {
        let _async: Token![async] = input.parse()?;
        let ident: Ident = input.parse()?;
        return Ok(GuardExpression::Guard(AsyncIdent {
            ident,
            is_async: true,
        }));
    }

    let ident: Ident = input.parse()?;
    Ok(GuardExpression::Guard(AsyncIdent {
        ident,
        is_async: false,
    }))
}

#[cfg(test)]
mod test {
    use crate::parser::transition::{
        GuardExpression, StateTransitions, EVENTLESS_NO_TARGET, EVENTLESS_WILDCARD,
    };
    use syn::parse_str;

    #[test]
    fn bad_guard_expression() {
        let guard_expression = "a && b c";
        assert!(parse_str::<GuardExpression>(guard_expression).is_err());
    }
    #[test]
    fn guard_expressions() -> Result<(), syn::Error> {
        for (guard_expression_str, expected) in vec![
            ("guard", "guard()"),
            ("async guard", "guard().await"),
            ("async a || async b", "a().await || b().await"),
            ("!guard", "!guard()"),
            ("a && b", "a() && b()"),
            ("a || b", "a() || b()"),
            ("a || b || c", "a() || b() || c()"),
            ("a || b && c || d", "a() || b() && c() || d()"),
            ("(a || b) && (c || d)", "(a() || b()) && (c() || d())"),
            ("a && b || c && d", "a() && b() || c() && d()"),
            (
                "a && ( !b && c ) || d && e",
                "a() && (!b() && c()) || d() && e()",
            ),
        ] {
            let guard_expression: GuardExpression = parse_str(guard_expression_str)?;
            assert_eq!(guard_expression.to_string(), expected);
            println!("{:?}", guard_expression);
        }
        Ok(())
    }

    fn parse_transitions(source: &str) -> syn::Result<StateTransitions> {
        parse_str::<StateTransitions>(source)
    }

    #[test]
    fn eventless_transitions_are_accepted() -> Result<(), syn::Error> {
        for source in [
            // unguarded
            "A = B",
            // guarded
            "A [g] = B",
            "A [ !g && h ] = B",
            // with action
            "A / a = B",
            "A [g] / a = B",
            // async guard and action
            "A [async g] / async a = B",
            // state data on the input state and the output state
            "A(u32) [g] = B",
            "A(u32) [async g] / async a = B",
            "A [g] / a = B(u32)",
            // self-loop
            "A [g] / a = A",
            // input state patterns
            "A | B = C",
            "A | B [g] / a = C",
            // The starting state is rejected later, when the whole machine is checked
            "*A / a = B",
        ] {
            let transitions = parse_transitions(source)?;
            assert!(
                transitions.event.is_none(),
                "`{}` should be eventless",
                source
            );
            assert!(!transitions.out_state.internal_transition, "`{}`", source);
        }
        Ok(())
    }

    #[test]
    fn eventless_transition_parts() -> Result<(), syn::Error> {
        let t = parse_transitions("Measuring(u32) [async ready] / async finish = Done")?;
        assert!(t.event.is_none());
        assert_eq!(t.in_states.len(), 1);
        assert_eq!(t.in_states[0].ident, "Measuring");
        assert!(t.in_states[0].data_type.is_some());
        assert_eq!(t.guard.as_ref().unwrap().to_string(), "ready().await");
        let action = t.action.as_ref().unwrap();
        assert_eq!(action.ident, "finish");
        assert!(action.is_async);
        assert_eq!(t.out_state.ident, "Done");

        let t = parse_transitions("A | B = C")?;
        assert!(t.event.is_none());
        assert_eq!(t.in_states.len(), 2);
        assert!(t.guard.is_none());
        assert!(t.action.is_none());
        Ok(())
    }

    #[test]
    fn event_transitions_still_have_an_event() -> Result<(), syn::Error> {
        for source in [
            "A + E = B",
            "A + E(u8) [g] / a = B",
            "A | B + E = C",
            "_ + E = C",
            // internal transitions are still fine with an event
            "A + E = _",
            "A + E / a = _",
            "A + E",
            "A + E [g]",
        ] {
            let transitions = parse_transitions(source)?;
            assert!(
                transitions.event.is_some(),
                "`{}` should have an event",
                source
            );
        }
        let t = parse_transitions("A + E(u8) = B")?;
        let event = t.event.unwrap();
        assert_eq!(event.ident, "E");
        assert!(event.data_type.is_some());
        Ok(())
    }

    #[test]
    fn eventless_wildcard_is_rejected() {
        for source in ["_ = A", "_ [g] = A", "_ / a = A", "_ [g] / a = A"] {
            let error = parse_transitions(source).unwrap_err();
            assert_eq!(error.to_string(), EVENTLESS_WILDCARD, "`{}`", source);
        }
        // A wildcard with an event is unchanged
        assert!(parse_transitions("_ + E = A").is_ok());
    }

    #[test]
    fn eventless_without_target_is_rejected() {
        for source in [
            // target omitted
            "A",
            "A / a",
            "A [g]",
            "A [g] / a",
            "A [async g] / async a",
            "A | B / a",
            // `A E = B` forgot the `+`
            "A E = B",
            // internal transition syntax
            "A = _",
            "A / a = _",
            "A [g] = _",
            "A [g] / a = _",
            "A | B [g] = _",
        ] {
            let error = parse_transitions(source).unwrap_err();
            assert_eq!(error.to_string(), EVENTLESS_NO_TARGET, "`{}`", source);
        }
    }

    #[test]
    fn eventless_wildcard_with_pattern_keeps_pattern_error() {
        let error = parse_transitions("_ | A = B").unwrap_err();
        assert_eq!(
            error.to_string(),
            "Wildcards already include all states, so should not be used with input state patterns."
        );
    }
}
