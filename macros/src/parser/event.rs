use crate::parser::transition::{GuardExpression, StateTransition};
use crate::parser::AsyncIdent;
use syn::{parenthesized, parse, spanned::Spanned, token, Ident, Token, Type};

#[derive(Debug, Clone)]
pub struct Event {
    pub ident: Ident,
    pub data_type: Option<Type>,
}

#[derive(Debug)]
pub struct EventMapping {
    pub event: Ident,
    pub transitions: Vec<Transition>,
}

#[derive(Debug)]
pub struct Transition {
    pub guard: Option<GuardExpression>,
    pub action: Option<AsyncIdent>,
    pub out_state: Ident,
}

impl From<&StateTransition> for Transition {
    fn from(transition: &StateTransition) -> Self {
        Self {
            guard: transition.guard.clone(),
            action: transition.action.clone(),
            out_state: transition.out_state.ident.clone(),
        }
    }
}

impl parse::Parse for Event {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        // Event
        input.parse::<Token![+]>()?;
        let ident: Ident = input.parse()?;

        // Possible type on the event
        let data_type = if input.peek(token::Paren) {
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

        Ok(Self { ident, data_type })
    }
}
