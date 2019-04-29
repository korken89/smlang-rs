#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;

mod parser;

//use parser::*;
//
use std::vec;

use quote::{quote, TokenStreamExt};
use syn::parse_macro_input;

// use std::collections::HashMap;

#[proc_macro]
pub fn statemachine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the syntax into structures
    let input = parse_macro_input!(input as parser::StateMachine);
    let sm = parser::ParsedStateMachine::new(input);
    //println!("ParsedStateMachine: {:#?}", sm);

    let s = &sm;
    // Get only the unique states
    let states = s.states.iter().map(|(_, value)| value);

    // Extract events
    let events = s.events.iter().map(|(_, value)| value);

    // Build the states and events output
    let mut output = quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum States { #(#states),* }

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Events { #(#events),* }

        pub enum Error { InvalidEvent, }
    };

    // Build the state machine runner
    let sm_code = quote! {
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
    }

    // How it should look when generated
    //
    // impl StateMachine {
    //    pub fn run(&mut self, event: Events) {
    //         match self.state {
    //             States::State1 => match event {
    //                 Events::Event1 => {
    //                     println!("State1, Event1"); // Do something real in the future
    //                     if guard1() {
    //                         action1();
    //                         self.state = States::State2;
    //                     }
    //                 }
    //                 _ => println!("State1, {:?}, nothing happens", event),
    //             },
    //         }
    //    }
    // }
    };

    output.append_all(sm_code);

    let transitions = &s.states_events_mapping;
    let in_states: vec::Vec<_> = transitions
        .iter()
        .map(|(key, _)| sm.states.get(key).unwrap())
        .collect();

    let events: vec::Vec<vec::Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.event).collect())
        .collect();

    // Merge guards, actions and output states into code blocks
    let guards: vec::Vec<vec::Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.guard).collect())
        .collect();

    let actions: vec::Vec<vec::Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.action).collect())
        .collect();

    let out_states: vec::Vec<vec::Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.out_state).collect())
        .collect();

    let code_blocks: vec::Vec<vec::Vec<_>> = guards
        .iter()
        .zip(actions.iter().zip(out_states.iter()))
        .map(|(guards, (actions, out_states))| {
            guards
                .iter()
                .zip(actions.iter().zip(out_states.iter()))
                .map(|(guard, (action, out_state))| {
                    if let Some(g) = guard {
                        if let Some(a) = action {
                            quote! {
                            if #g() {
                                #a();
                                self.state = States::#out_state;
                            }
                            }
                        } else {
                            quote! {
                            if #g() {
                                self.state = States::#out_state;
                            }
                            }
                        }
                    } else {
                        if let Some(a) = action {
                            quote! {
                                #a();
                                self.state = States::#out_state;
                            }
                        } else {
                            quote! {
                                self.state = States::#out_state;
                            }
                        }
                    }
                })
                .collect()
        })
        .collect();

    // Create the code blocks inside the switch cases

    //let test: std::vec::Vec<_> = test.iter().map(|(_, value)| value).collect();
    //println!("Transitions: {:#?}", transitions);
    // println!("States: {:#?}", in_states);
    // println!("Events: {:#?}", events);
    // println!("Guards: {:#?}", guards);
    // println!("Actions: {:#?}", actions);
    // println!("Out states: {:#?}", out_states);
    // println!("Code blocks: {:#?}", code_blocks);

    // Combine states, events and the internals code blocks
    let test = quote! {

    impl StateMachine {
        pub fn run(&mut self, event: Events) -> Result<(),Error> {
            //println!("In state: {:?}", self.state);

            match self.state {
                #(States::#in_states => match event {
                    #(Events::#events => {
                        #code_blocks

                        //println!("Going to state: {:?}", self.state);
                        //println!("");
                        Ok(())
                    }),*
                    _ => {
                        //println!("State1, {:?}, nothing happens", event);
                        //println!("");
                        Err(Error::InvalidEvent)
                    },
                }),*
            }
        }
    }
    };

    //println!("Test output: {:#?}", test.to_string());

    output.append_all(test);

    // Hand the output tokens back to the compiler
    output.into()
}
