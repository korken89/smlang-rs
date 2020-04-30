// Move guards to return a Result

use crate::parser::*;
use proc_macro2;
use proc_macro2::Span;
use quote::quote;
use std::vec::Vec;
use syn::{punctuated::Punctuated, token::Paren, Type, TypeTuple};

pub fn generate_code(sm: &ParsedStateMachine) -> proc_macro2::TokenStream {
    // Get only the unique states
    let mut state_list: Vec<_> = sm.states.iter().map(|(_, value)| value).collect();
    state_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    let state_list: Vec<_> = state_list
        .iter()
        .map(|value| match sm.state_data_type.get(&value.to_string()) {
            None => {
                quote! {
                    #value
                }
            }
            Some(t) => {
                quote! {
                    #value(#t)
                }
            }
        })
        .collect();

    // Extract events
    let mut event_list: Vec<_> = sm.events.iter().map(|(_, value)| value).collect();
    event_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    // Extract events
    let event_list: Vec<_> = event_list
        .iter()
        .map(|value| match sm.event_data_type.get(&value.to_string()) {
            None => {
                quote! {
                    #value
                }
            }
            Some(t) => {
                quote! {
                    #value(#t)
                }
            }
        })
        .collect();

    let transitions = &sm.states_events_mapping;

    let in_states: Vec<_> = transitions
        .iter()
        .map(|(name, _)| {
            let state_name = sm.states.get(name).unwrap();

            match sm.state_data_type.get(name) {
                None => {
                    quote! {
                        #state_name
                    }
                }
                Some(_) => {
                    quote! {
                        #state_name(ref state_data)
                    }
                }
            }
        })
        .collect();

    let events: Vec<Vec<_>> = transitions
        .iter()
        .map(|(_, value)| {
            value
                .iter()
                .map(|(name, value)| {
                    let value = &value.event;

                    match sm.event_data_type.get(name) {
                        None => {
                            quote! {
                                #value
                            }
                        }
                        Some(_) => {
                            quote! {
                                #value(ref event_data)
                            }
                        }
                    }
                })
                .collect()
        })
        .collect();

    // println!("sm: {:#?}", sm);
    // println!("in_states: {:#?}", in_states);
    // println!("events: {:#?}", events);
    // println!("transitions: {:#?}", transitions);

    // Map guards, actions and output states into code blocks
    let guards: Vec<Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.guard).collect())
        .collect();

    let actions: Vec<Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.action).collect())
        .collect();

    let guard_action_parameters: Vec<Vec<_>> = transitions
        .iter()
        .map(|(name, value)| {
            let state_name = &sm.states.get(name).unwrap().to_string();

            value
                .iter()
                .map(|(name, _)| {
                    // let event_name = &value.event;

                    match (
                        sm.state_data_type.get(state_name),
                        sm.event_data_type.get(name),
                    ) {
                        (None, None) => {
                            quote! {}
                        }
                        (Some(_), None) => {
                            quote! {
                                state_data
                            }
                        }
                        (None, Some(_)) => {
                            quote! {
                                event_data
                            }
                        }
                        (Some(_), Some(_)) => {
                            quote! {
                                state_data, event_data
                            }
                        }
                    }
                })
                .collect()
        })
        .collect();

    let out_states: Vec<Vec<_>> = transitions
        .iter()
        .map(|(_, value)| {
            value
                .iter()
                .map(|(_, value)| {
                    let out_state = &value.out_state;

                    match sm.state_data_type.get(&out_state.to_string()) {
                        None => {
                            quote! {
                                #out_state
                            }
                        }
                        Some(_) => {
                            quote! {
                                #out_state(_data)
                            }
                        }
                    }
                })
                .collect()
        })
        .collect();

    let mut g2 = proc_macro2::TokenStream::new();
    let mut a2 = proc_macro2::TokenStream::new();
    for (state, value) in transitions.iter() {
        value.iter().for_each(|(event, value)| {
            // Create the guard traits for user implementation
            if let Some(guard) = &value.guard {
                match (sm.state_data_type.get(state), sm.event_data_type.get(event)) {
                    (None, None) => {
                        g2.extend(quote! {
                            fn #guard(&self) -> bool;
                        });
                    }
                    (Some(st), None) => {
                        g2.extend(quote! {
                            fn #guard(&self, state_data: &#st) -> bool;
                        });
                    }
                    (None, Some(et)) => {
                        g2.extend(quote! {
                            fn #guard(&self, event_data: &#et) -> bool;
                        });
                    }
                    (Some(st), Some(et)) => {
                        g2.extend(quote! {
                            fn #guard(&self, state_data: &#st, event_data: &#et) -> bool;
                        });
                    }
                }
            }

            // Create the action traits for user implementation
            if let Some(action) = &value.action {
                let return_type = if let Some(output_data) =
                    sm.state_data_type.get(&value.out_state.to_string())
                {
                    output_data.clone()
                } else {
                    // Empty return type
                    Type::Tuple(TypeTuple {
                        paren_token: Paren {
                            span: Span::call_site(),
                        },
                        elems: Punctuated::new(),
                    })
                };

                match (sm.state_data_type.get(state), sm.event_data_type.get(event)) {
                    (None, None) => {
                        a2.extend(quote! {
                            fn #action(&mut self) -> #return_type;
                        });
                    }
                    (Some(st), None) => {
                        a2.extend(quote! {
                            fn #action(&mut self, state_data: &#st) -> #return_type;
                        });
                    }
                    (None, Some(et)) => {
                        a2.extend(quote! {
                            fn #action(&mut self, event_data: &#et) -> #return_type;
                        });
                    }
                    (Some(st), Some(et)) => {
                        a2.extend(quote! {
                            fn #action(&mut self, state_data: &#st, event_data: &#et) -> #return_type;
                        });
                    }
                }
            }
        })
    }

    // Create the code blocks inside the switch cases
    let code_blocks: Vec<Vec<_>> = guards
        .iter()
        .zip(
            actions
                .iter()
                .zip(out_states.iter().zip(guard_action_parameters.iter())),
        )
        .map(
            |(guards, (actions, (out_states, guard_action_parameters)))| {
                guards
                    .iter()
                    .zip(
                        actions
                            .iter()
                            .zip(out_states.iter().zip(guard_action_parameters.iter())),
                    )
                    .map(|(guard, (action, (out_state, g_a_param)))| {
                        if let Some(g) = guard {
                            if let Some(a) = action {
                                quote! {
                                    if self.context.#g(#g_a_param) {
                                        let _data = self.context.#a(#g_a_param);
                                        self.state = States::#out_state;
                                    } else {
                                        return Err(Error::GuardFailed);
                                    }
                                }
                            } else {
                                quote! {
                                    if self.context.#g(#g_a_param) {
                                        self.state = States::#out_state;
                                    } else {
                                        return Err(Error::GuardFailed);
                                    }
                                }
                            }
                        } else {
                            if let Some(a) = action {
                                quote! {
                                    let _data = self.context.#a(#g_a_param);
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
            },
        )
        .collect();

    let starting_state = &sm.starting_state;

    // Build the states and events output
    quote! {
        pub trait StateMachineContext : core::fmt::Debug {
            #g2
            #a2
        }

        /// List of auto-generated states
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub enum States { #(#state_list),* }

        /// List of auto-generated events
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub enum Events { #(#event_list),* }

        /// List of possible errors
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Error {
            /// This can happen when an event is processed which should not come in this stage
            /// of processing
            InvalidEvent,
            /// This can happen when an event is processed whose guard did not return `true`
            GuardFailed,
        }

        /// State machine structure definition
        #[derive(Debug)]
        pub struct StateMachine<T: StateMachineContext> {
            state: States,
            context: T
        }

        impl<T: StateMachineContext> StateMachine<T> {
            /// Creates a new state machine with the specified starting state
            pub fn new(context: T) -> Self {
                StateMachine {
                    state: States::#starting_state,
                    context
                }
            }

            /// Returns the current state
            pub fn state(&self) -> States {
                self.state
            }

            /// Returns the current context as a reference
            pub fn context(&self) -> &T {
                &self.context
            }

            /// Returns the current context as a mutable reference
            pub fn context_mut(&mut self) -> &mut T {
                &mut self.context
            }

            /// Process an event
            ///
            /// It will return `Ok(NextState)` if the transition was successful, or `Err(Error)`
            /// if there was an error in the transition
            pub fn process_event(&mut self, event: Events) -> Result<States, Error> {
                match self.state {
                    #(States::#in_states => match event {
                        #(Events::#events => {
                            #code_blocks

                            Ok(self.state)
                        }),*
                        _ => Err(Error::InvalidEvent),
                    }),*
                    _ => Err(Error::InvalidEvent),
                }
            }
        }
    }
}
