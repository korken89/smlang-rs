use super::input_state::InputState;

use syn::{bracketed, parenthesized, parse, spanned::Spanned, token, Ident, Token, Type};

#[derive(Debug)]
pub struct StateTransition {
    pub in_state: InputState,
    pub event: Ident,
    pub event_data_type: Option<Type>,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
    pub out_state: Ident,
    pub out_state_data_type: Option<Type>,
}

#[derive(Debug)]
pub struct StateTransitions {
    pub in_states: Vec<InputState>,
    pub event: Ident,
    pub event_data_type: Option<Type>,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
    pub out_state: Ident,
    pub out_state_data_type: Option<Type>,
}

impl parse::Parse for StateTransitions {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        // parse the input pattern
        let mut in_states = Vec::new();
        loop {
            let in_state: InputState = input.parse()?;
            in_states.push(in_state);
            if let Err(_) = input.parse::<Token![|]>() {
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
        input.parse::<Token![+]>()?;
        let event: Ident = input.parse()?;

        // Possible type on the event
        let event_data_type = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let input: Type = content.parse()?;

            // Check so the type is supported
            match &input {
                Type::Array(_)
                | Type::Path(_)
                | Type::Ptr(_)
                | Type::Reference(_)
                | Type::Slice(_)
                | Type::Tuple(_) => (),
                _ => {
                    return Err(parse::Error::new(
                        input.span(),
                        "This is an unsupported type for events.",
                    ))
                }
            }

            Some(input)
        } else {
            None
        };

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
        let action = if let Ok(_) = input.parse::<Token![/]>() {
            let action: Ident = input.parse()?;
            Some(action)
        } else {
            None
        };

        input.parse::<Token![=]>()?;

        let out_state: Ident = input.parse()?;

        // Possible type on the output state
        let out_state_data_type = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let input: Type = content.parse()?;

            // Check so the type is supported
            match &input {
                Type::Array(_)
                | Type::Path(_)
                | Type::Ptr(_)
                | Type::Reference(_)
                | Type::Slice(_)
                | Type::Tuple(_) => (),
                _ => {
                    return Err(parse::Error::new(
                        input.span(),
                        "This is an unsupported type for states.",
                    ))
                }
            }

            Some(input)
        } else {
            None
        };

        Ok(Self {
            in_states,
            event,
            event_data_type,
            guard,
            action,
            out_state,
            out_state_data_type,
        })
    }
}
