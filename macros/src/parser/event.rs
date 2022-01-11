use syn::Ident;

#[derive(Debug)]
pub struct EventMapping {
    pub event: Ident,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
    pub out_state: Ident,
}
