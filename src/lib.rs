#![recursion_limit = "128"]

extern crate proc_macro;

mod codegen;
mod parser;
use syn::parse_macro_input;

// use std::collections::HashMap;

#[proc_macro]
pub fn statemachine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the syntax into structures
    let input = parse_macro_input!(input as parser::StateMachine);

    let sm = parser::ParsedStateMachine::new(input);

    let output = codegen::generate_code(&sm);

    // Hand the output tokens back to the compiler
    output.into()
}
