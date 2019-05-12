use crate::parser::*;
use proc_macro2;
use quote::quote;
use std::vec;
use std::collections::HashSet;

pub fn generate_code(sm: &ParsedStateMachine) -> proc_macro2::TokenStream {
    // Get only the unique states
    let mut state_list: vec::Vec<_> = sm.states.iter().map(|(_, value)| value).collect();
    state_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    // Extract events
    let mut event_list: vec::Vec<_> = sm.events.iter().map(|(_, value)| value).collect();
    event_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    let transitions = &sm.states_events_mapping;
    let in_states: vec::Vec<_> = transitions
        .iter()
        .map(|(key, _)| sm.states.get(key).unwrap())
        .collect();

    let events: vec::Vec<vec::Vec<_>> = transitions
        .iter()
        .map(|(_, value)| value.iter().map(|(_, value)| &value.event).collect())
        .collect();

    // Map guards, actions and output states into code blocks
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

    let guard_context_methods: HashSet<_> = guards
        .iter()
        .flatten()
        .filter_map(|g| g.as_ref())
        .collect();

    let action_context_methods: HashSet<_> = actions
        .iter()
        .flatten()
        .filter_map(|a| a.as_ref())
        .collect();

    // Create the code blocks inside the switch cases
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
                                if self.context.#g() {
                                    self.context.#a();
                                    self.state = States::#out_state;
                                } else {
                                    return Err(Error::GuardFailed);
                                }
                            }
                        } else {
                            quote! {
                                if self.context.#g() {
                                    self.state = States::#out_state;
                                } else {
                                    return Err(Error::GuardFailed);
                                }
                            }
                        }
                    } else {
                        if let Some(a) = action {
                            quote! {
                                self.context.#a();
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
        pub trait StateMachineContext : Default + core::fmt::Debug {
            #(fn #guard_context_methods(&self) -> bool;)*
            #(fn #action_context_methods(&self);)*
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
            pub fn new() -> Self {
                StateMachine {
                    state: States::#starting_state,
                    context: T::default()
                }
            }

            /// Returns the current state
            pub fn state(&self) -> States {
                self.state
            }

            /// Process an event
            ///
            /// It will return `Ok(NextState)` if the transition was successful, or `Err(Error::...)`
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
