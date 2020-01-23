#![recursion_limit = "512"]

extern crate proc_macro;

mod codegen;
mod parser;

use syn::parse_macro_input;

#[proc_macro]
pub fn statemachine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the syntax into structures
    let input = parse_macro_input!(input as parser::StateMachine);

    // Validate syntax
    match parser::ParsedStateMachine::new(input) {
        // Generate code and hand the output tokens back to the compiler
        Ok(sm) => codegen::generate_code(&sm).into(),
        Err(error) => error.to_compile_error().into(),
    }
}
