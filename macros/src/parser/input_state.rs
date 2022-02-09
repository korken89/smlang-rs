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

        // wildcards can't be used as starting states
        if start && wildcard {
            return Err(parse::Error::new(
                input.span(),
                "Wildcards can't be used as the starting state.",
            ));
        }

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

#[cfg(test)]
mod tests {

    use super::*;
    use syn::parse_quote;

    #[test]
    #[should_panic(expected = "Wildcards can't be used as the starting state.")]
    fn wildcard_used_as_start() {
        let _: InputState = parse_quote! {
            *_
        };
    }

    #[test]
    fn input_state_with_data() {
        let state: InputState = parse_quote! {
            *Start(u8)
        };

        assert!(state.start);
        assert!(!state.wildcard);
        assert!(state.data_type.is_some());
    }

    #[test]
    #[should_panic(expected = "Wildcard states cannot have data associated with it.")]
    fn wildcard_with_data() {
        let _: InputState = parse_quote! {
            _(u8)
        };
    }

    #[test]
    #[should_panic(expected = "This is an unsupported type for states.")]
    fn unsupported_type() {
        let _: InputState = parse_quote! {
            State1(!)
        };
    }

    #[test]
    fn wildcard() {
        let wildcard: InputState = parse_quote! {
            _
        };

        assert!(wildcard.wildcard);
        assert!(!wildcard.start);
        assert!(wildcard.data_type.is_none());
    }

    #[test]
    fn start() {
        let start: InputState = parse_quote! {
            *Start
        };

        assert!(start.start);
        assert!(!start.wildcard);
        assert!(start.data_type.is_none());
    }

    #[test]
    fn state_without_data() {
        let state: InputState = parse_quote! {
            State
        };

        assert!(!state.start);
        assert!(!state.wildcard);
        assert!(state.data_type.is_none());
    }

    #[test]
    fn state_with_data() {
        let state: InputState = parse_quote! {
            State(u8)
        };

        assert!(!state.start);
        assert!(!state.wildcard);
        assert!(state.data_type.is_some());
    }
}
