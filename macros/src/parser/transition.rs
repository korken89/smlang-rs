use super::event::Event;
use super::input_state::InputState;
use super::output_state::OutputState;
use super::AsyncIdent;
use proc_macro2::TokenStream;
use quote::quote;
use std::fmt;
use syn::{bracketed, parse, token, Ident, Token};

#[derive(Debug)]
pub struct StateTransition {
    pub in_state: InputState,
    pub event: Event,
    pub guard: Option<GuardExpression>,
    pub action: Option<AsyncIdent>,
    pub out_state: OutputState,
}

#[derive(Debug)]
pub struct StateTransitions {
    pub in_states: Vec<InputState>,
    pub event: Event,
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

        // Event
        let event: Event = input.parse()?;

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

        let out_state: OutputState = input.parse()?;

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
    pub fn to_token_stream<F>(&self, context: &mut F) -> TokenStream
    where
        F: FnMut(&AsyncIdent) -> TokenStream,
    {
        match self {
            GuardExpression::Guard(async_ident) => async_ident.to_token_stream(context),
            GuardExpression::Not(expr) => {
                let expr_tokens = expr.to_token_stream(context);
                quote! { !#expr_tokens }
            }
            GuardExpression::Group(expr) => {
                let expr_tokens = expr.to_token_stream(context);
                quote! { (#expr_tokens) }
            }
            GuardExpression::And(lhs, rhs) => {
                let lhs_tokens = lhs.to_token_stream(context);
                let rhs_tokens = rhs.to_token_stream(context);
                quote! { #lhs_tokens && #rhs_tokens }
            }
            GuardExpression::Or(lhs, rhs) => {
                let lhs_tokens = lhs.to_token_stream(context);
                let rhs_tokens = rhs.to_token_stream(context);
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
    use crate::parser::transition::GuardExpression;
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
}
