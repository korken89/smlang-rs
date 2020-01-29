use crate::parser::*;
use proc_macro2;
use quote::quote;
use std::vec::Vec;

pub fn generate_code(sm: &ParsedStateMachine) -> proc_macro2::TokenStream {
    // Get only the unique states
    let mut state_list: Vec<_> = sm.states.iter().map(|(_, value)| value).collect();
    state_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

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
        .map(|(key, _)| sm.states.get(key).unwrap())
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
                                #value(ref data)
                            }
                        }
                    }
                })
                .collect()
        })
        .collect();

    // println!("sm: {:#?}", sm);
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

    let out_states: Vec<Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.out_state).collect())
        .collect();

    let mut guard_context_methods: Vec<_> =
        guards.iter().flatten().filter_map(|g| g.as_ref()).collect();
    guard_context_methods.dedup();

    let mut g2 = Vec::new();
    for (_state, value) in transitions.iter() {
        value.iter().for_each(|(event, value)| {
            let guard = &value.guard;
            g2.push(match sm.event_data_type.get(event) {
                None => {
                    quote! {
                        fn #guard(&self) -> bool;
                    }
                }
                Some(t) => {
                    quote! {
                        fn #guard(&self, data: &#t) -> bool;
                    }
                }
            });
        })
    }

    println!("new guard: {:#?}", g2);

    let mut action_context_methods: Vec<_> = actions
        .iter()
        .flatten()
        .filter_map(|a| a.as_ref())
        .collect();
    action_context_methods.dedup();

    // Create the code blocks inside the switch cases
    let code_blocks: Vec<Vec<_>> = guards
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
                                if self.context.#g(&event) {
                                    self.context.#a(&event);
                                    self.state = States::#out_state;
                                } else {
                                    return Err(Error::GuardFailed);
                                }
                            }
                        } else {
                            quote! {
                                if self.context.#g(&event) {
                                    self.state = States::#out_state;
                                } else {
                                    return Err(Error::GuardFailed);
                                }
                            }
                        }
                    } else {
                        if let Some(a) = action {
                            quote! {
                                self.context.#a(&event);
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

    let starting_state = &sm.starting_state;

    // Build the states and events output
    quote! {
        pub trait StateMachineContext : core::fmt::Debug {
            //#(#(#g2)*)*
            // #(fn #action_context_methods(&mut self, event: &Events);)*
        }

        /// List of auto-generated states
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum States { #(#state_list),* }

        /// List of auto-generated events
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
