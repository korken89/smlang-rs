use syn::{parenthesized, parse, spanned::Spanned, token, Ident, Token, Type};

#[derive(Debug, Clone)]
pub struct OutputState {
    pub ident: Ident,
    pub data_type: Option<Type>,
}

impl parse::Parse for OutputState {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![=]>()?;
        let ident: Ident = input.parse()?;

        // Possible type on the output state
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
                        "This is an unsupported type for states.",
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
