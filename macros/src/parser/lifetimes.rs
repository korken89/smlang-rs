use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::ops::Sub;
use syn::{parse, spanned::Spanned, GenericArgument, Lifetime, PathArguments, Type};

#[derive(Default, Debug, Clone)]
pub struct Lifetimes {
    lifetimes: Vec<Lifetime>,
}

impl Lifetimes {
    pub fn new() -> Lifetimes {
        Lifetimes {
            lifetimes: Vec::new(),
        }
    }

    pub fn from_type(data_type: &Type) -> Result<Lifetimes, parse::Error> {
        let mut lifetimes = Lifetimes::new();
        lifetimes.insert_from_type(data_type)?;
        Ok(lifetimes)
    }

    pub fn insert(&mut self, lifetime: &Lifetime) {
        if !self.lifetimes.contains(lifetime) {
            self.lifetimes.push(lifetime.to_owned());
        }
    }

    pub fn extend(&mut self, other: &Lifetimes) {
        for lifetime in other.lifetimes.iter() {
            self.insert(lifetime);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lifetimes.is_empty()
    }

    pub fn as_slice(&self) -> &[Lifetime] {
        &self.lifetimes[..]
    }

    /// Extracts lifetimes from a [`Type`]
    pub fn insert_from_type(&mut self, data_type: &Type) -> Result<(), parse::Error> {
        match data_type {
            Type::Reference(tr) => {
                if let Some(lifetime) = &tr.lifetime {
                    self.insert(lifetime);
                } else {
                    return Err(parse::Error::new(
                        data_type.span(),
                        "This event's data lifetime is not defined, consider adding a lifetime.",
                    ));
                }
            }
            Type::Path(tp) => {
                let punct = &tp.path.segments;
                for p in punct.iter() {
                    if let PathArguments::AngleBracketed(abga) = &p.arguments {
                        for arg in &abga.args {
                            if let GenericArgument::Lifetime(lifetime) = &arg {
                                self.insert(lifetime);
                            }
                        }
                    }
                }
            }
            Type::Tuple(tuple) => {
                for elem in tuple.elems.iter() {
                    self.insert_from_type(elem)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl ToTokens for Lifetimes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.is_empty() {
            return;
        }

        let lifetimes = self.as_slice();
        tokens.extend(quote! { #(#lifetimes),* ,});
    }
}

impl Sub<&Lifetimes> for Lifetimes {
    type Output = Lifetimes;

    fn sub(mut self, rhs: &Lifetimes) -> Lifetimes {
        self.lifetimes.retain(|lt| !rhs.lifetimes.contains(lt));
        self
    }
}

impl Sub for Lifetimes {
    type Output = Lifetimes;

    fn sub(self, rhs: Lifetimes) -> Lifetimes {
        self.sub(&rhs)
    }
}

impl Sub<&Lifetimes> for &Lifetimes {
    type Output = Lifetimes;

    fn sub(self, rhs: &Lifetimes) -> Lifetimes {
        self.to_owned().sub(rhs)
    }
}

impl Sub<Lifetimes> for &Lifetimes {
    type Output = Lifetimes;

    fn sub(self, rhs: Lifetimes) -> Lifetimes {
        self.to_owned().sub(&rhs)
    }
}
