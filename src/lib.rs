extern crate proc_macro;
extern crate syn;

mod parser;

//use parser::*;

use quote::{quote, TokenStreamExt};
use syn::parse_macro_input;

// use std::collections::HashMap;

#[proc_macro]
pub fn statemachine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the syntax into structures
    let input = parse_macro_input!(input as parser::StateMachine);
    let sm = parser::ParsedStateMachine::new(input);
    //println!("ParsedStateMachine: {:#?}", sm);

    // Get only the unique states
    let states = sm.states.iter().map(|(_, value)| value);

    // Extract events
    let events = sm.events.iter().map(|(_, value)| value);

    // Build the states and events output
    let mut output = quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum States { #(#states),* }

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Events { #(#events),* }
    };

    // Build the state machine runner
    let sm = quote! {
    struct StateMachine {
        state: States,
    }

    impl StateMachine {
        pub fn new() -> Self {
            StateMachine {
                state: States::State1,
            }
        }

        pub fn state(&self) -> States {
            self.state
        }

        pub fn run(&mut self, event: Events) {
            // match self.state {
            //     States::State1 => match event {
            //         Events::Event1 => {
            //             println!("State1, Event1"); // Do something real in the future
            //             if guard1() {
            //                 action1();
            //                 self.state = States::State2;
            //             }
            //         }
            //         _ => println!("State1, {:?}, nothing happens", event),
            //     },
            // }
        }
    }
    };

    output.append_all(sm);

    let a = &vec!["a", "b"];
    let mut b = &vec![vec!["a1", "a2"], vec!["b1", "b2"]];
    //b.clear();

    let test = quote! {
    fn test() {
        println!(#(#a),*);
        println!(#(#a #(#b),*),*);
        println!(#(#(#b),*),*);
    }
    };

    println!("Test: {:#?}", test.to_string());

    // Hand the output tokens back to the compiler
    output.into()
}
