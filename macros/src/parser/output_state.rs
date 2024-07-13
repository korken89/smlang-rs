use proc_macro2::Span;
use syn::{parenthesized, parse, spanned::Spanned, token, Ident, Token, Type};

#[derive(Debug, Clone)]
pub struct OutputState {
    pub ident: Ident,
    pub internal_transition: bool,
    pub data_type: Option<Type>,
}

impl parse::Parse for OutputState {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let (internal_transition, ident) = if input.peek(Token![_]) {
                // Underscore ident here is used to represent an internal transition
                let underscore = input.parse::<Token![_]>()?;
                (true, underscore.into())
            } else {
                (false, input.parse()?)
            };

            // Possible type on the output state
            let data_type = if !internal_transition && input.peek(token::Paren) {
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
                ident,
                internal_transition,
                data_type,
            })
        } else {
            // Internal transition
            Ok(Self {
                ident: Ident::new("_", Span::call_site()),
                internal_transition: true,
                data_type: None,
            })
        }
    }
}
