use super::event::Event;
use super::input_state::InputState;
use super::output_state::OutputState;
use syn::{bracketed, parse, token, Ident, Token};

#[derive(Debug)]
pub struct StateTransition {
    pub in_state: InputState,
    pub event: Event,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
    pub out_state: OutputState,
}

#[derive(Debug)]
pub struct StateTransitions {
    pub in_states: Vec<InputState>,
    pub event: Event,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
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
            let guard: Ident = content.parse()?;
            Some(guard)
        } else {
            None
        };

        // Possible action
        let action = if input.parse::<Token![/]>().is_ok() {
            let action: Ident = input.parse()?;
            Some(action)
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
