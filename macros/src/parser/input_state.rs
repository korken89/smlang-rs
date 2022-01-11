use syn::{parenthesized, parse, spanned::Spanned, token, Ident, Token, Type};

#[derive(Debug, Clone)]
pub struct InputState {
    pub start: bool,
    pub wildcard: bool,
    pub ident: Ident,
    pub data_type: Option<Type>,
}

impl parse::Parse for InputState {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        // Check for starting state definition
        let start = input.parse::<Token![*]>().is_ok();

        // check to see if this is a wildcard state, which is denoted with "underscore"
        let underscore = input.parse::<Token![_]>();
        let wildcard = underscore.is_ok();

        // Input State
        let ident: Ident = if let Ok(underscore) = underscore {
            underscore.into()
        } else {
            input.parse()?
        };

        // Possible type on the input state
        let data_type = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let input: Type = content.parse()?;

            // Check if this is the starting state, it cannot have data as there is no
            // supported way of propagating it (for now)
            if start {
                return Err(parse::Error::new(
                    input.span(),
                    "The starting state cannot have data associated with it.",
                ));
            }

            // Wilcards should not have data associated, as data will already be defined
            if wildcard {
                return Err(parse::Error::new(
                    input.span(),
                    "Wildcard states cannot have data associated with it.",
                ));
            }

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
            start,
            wildcard,
            ident,
            data_type,
        })
    }
}
