#![recursion_limit = "512"]

extern crate proc_macro;

mod codegen;
mod diagramgen;
mod parser;

use syn::parse_macro_input;

// dot -Tsvg statemachine.gv -o statemachine.svg

#[proc_macro]
pub fn statemachine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the syntax into structures
    let input = parse_macro_input!(input as parser::StateMachine);

    // Validate syntax
    match parser::ParsedStateMachine::new(input) {
        // Generate code and hand the output tokens back to the compiler
        Ok(sm) => {
            use std::io::Write;
            
            // Generate dot syntax for the statemachine.
            let diagram = diagramgen::generate_diagram(&sm);

            #[cfg(feature = "graphviz")]
            {
                // Start the 'dot' process.
                let mut process = std::process::Command::new("dot")
                    .args(&[
                        "-Tsvg",
                        "-o",
                        "statemachine.svg",
                    ])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .expect("Failed to execute 'dot'. Are you sure graphviz is installed?");

                // Write the dot syntax string to the 'dot' process stdin.
                process
                    .stdin
                    .as_mut()
                    .map(|s| s.write_all(diagram.as_bytes()));
                
                // Check the graphviz return status to see if it was successful.
                match process.wait() {
                    Ok(status) => if !status.success() {
                        panic!("'dot' failed to run. Are you sure graphviz is installed?");
                    },
                    Err(_) => panic!("'dot' failed to run. Are you sure graphviz is installed?")
                }
            }
            #[cfg(not(feature = "graphviz"))]
            {
                // Try open a dot file.
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open("statemachine.gv") {
                        Ok(mut file) => {
                            // Try write to the dot file.
                            if let Err(err) = file.write_all(diagram.as_bytes()) {
                                // Fallback to printing to the commandline.
                                println!("'statemachine.dot' could not be created: {}", err);
                                println!("Here is the contents:");
                                println!("{}", diagram);
                            }
                        },
                        Err(err) => {
                            // Fallback to printing to the commandline.
                            println!("'statemachine.dot' could not be created: {}", err);
                            println!("Here is the contents:");
                            println!("{}", diagram);
                        }
                }
            }

            codegen::generate_code(&sm).into()
        },
        Err(error) => error.to_compile_error().into(),
    }
}
