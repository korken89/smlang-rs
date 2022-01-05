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
        .map(
            |value| match sm.state_data.data_types.get(&value.to_string()) {
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
            },
        )
        .collect();

    // Extract events
    let mut event_list: Vec<_> = sm.events.iter().map(|(_, value)| value).collect();
    event_list.sort_by(|a, b| a.to_string().cmp(&b.to_string()));

    // Extract events
    let event_list: Vec<_> = event_list
        .iter()
        .map(
            |value| match sm.event_data.data_types.get(&value.to_string()) {
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
            },
        )
        .collect();

    let transitions = &sm.states_events_mapping;

    let in_states: Vec<_> = transitions
        .iter()
        .map(|(name, _)| {
            let state_name = sm.states.get(name).unwrap();

            match sm.state_data.data_types.get(name) {
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

                    match sm.event_data.data_types.get(name) {
                        None => {
                            quote! {
                                #value
                            }
                        }
                        Some(_) => {
                            quote! {
                                #value(ref mut event_data)
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
                        sm.state_data.data_types.get(state_name),
                        sm.event_data.data_types.get(name),
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

                    match sm.state_data.data_types.get(&out_state.to_string()) {
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

    let temporary_context = match &sm.temporary_context_type {
        Some(tct) => {
            quote! { temporary_context: #tct, }
        }
        None => {
            quote! {}
        }
    };

    // Keep track of already added actions not to duplicate definitions
    let mut action_set: Vec<syn::Ident> = Vec::new();
    let mut guard_set: Vec<syn::Ident> = Vec::new();

    let mut guard_list = proc_macro2::TokenStream::new();
    let mut action_list = proc_macro2::TokenStream::new();
    for (state, value) in transitions.iter() {
        value.iter().for_each(|(event, value)| {
            // Create the guard traits for user implementation
            if let Some(guard) = &value.guard {
                let guard_with_lifetimes = if let Some(lifetimes) = sm.event_data.lifetimes.get(event) {
                    let lifetimes = &lifetimes;
                    quote! {
                        #guard<#(#lifetimes),*>
                    }
                } else {
                    quote! {
                        #guard
                    }
                };

                let state_data = match sm.state_data.data_types.get(state) {
                    Some(st) => {
                        quote! { state_data: &#st, }
                    }
                    None => {
                        quote! {}
                    }
                };
                let event_data = match sm.event_data.data_types.get(event) {
                    Some(et) => match et {
                        Type::Reference(_) => {
                            quote! { event_data: #et }
                        }
                        _ => {
                            quote! { event_data: &#et }
                        }
                    },
                    None => {
                        quote! {}
                    }
                };

                let guard_error = if let Some(ref guard_error) = sm.guard_error {
                    quote! { #guard_error }

                } else {
                    quote! { () }
                };

                // Only add the guard if it hasn't been added before
                if guard_set.iter().find(|a| a == &guard).is_none() {
                    guard_set.push(guard.clone());
                    guard_list.extend(quote! {
                        #[allow(missing_docs)]
                        fn #guard_with_lifetimes(&mut self, #temporary_context #state_data #event_data) -> Result<(), #guard_error>;
                    });
                }
            }

            // Create the action traits for user implementation
            if let Some(action) = &value.action {
                let return_type = if let Some(output_data) =
                    sm.state_data.data_types.get(&value.out_state.to_string())
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

                let action_with_lifetimes = if let Some(lifetimes) = sm.event_data.lifetimes.get(event) {
                    let lifetimes = &lifetimes;
                    quote! {
                        #action<#(#lifetimes),*>
                    }
                } else {
                    quote! {
                        #action
                    }
                };

                let state_data = match sm.state_data.data_types.get(state) {
                    Some(st) => {
                        quote! { state_data: &#st, }
                    }
                    None => {
                        quote! {}
                    }
                };
                let event_data = match sm.event_data.data_types.get(event) {
                    Some(et) => match et {
                        Type::Reference(_) => {
                            quote! { event_data: #et }
                        }
                        _ => {
                            quote! { event_data: &#et }
                        }
                    },
                    None => {
                        quote! {}
                    }
                };

                // Only add the action if it hasn't been added before
                if action_set.iter().find(|a| a == &action).is_none() {
                    action_set.push(action.clone());
                    action_list.extend(quote! {
                        #[allow(missing_docs)]
                        fn #action_with_lifetimes(&mut self, #temporary_context #state_data #event_data) -> #return_type;
                    });
                }
            }
        })
    }

    let temporary_context_call = match &sm.temporary_context_type {
        Some(_) => {
            quote! { temporary_context, }
        }
        None => {
            quote! {}
        }
    };

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
                                    match self.context.#g(#temporary_context_call #g_a_param) {
                                        Ok(()) => {
                                            let _data = self.context.#a(#temporary_context_call #g_a_param);
                                            self.state = States::#out_state;
                                        },
                                        Err(e) => {
                                            return Err(Error::GuardFailed(e));
                                        }
                                    }
                                }
                            } else {
                                quote! {
                                    match self.context.#g(#temporary_context_call #g_a_param) {
                                        Ok(()) => {
                                            self.state = States::#out_state;
                                        },
                                        Err(e) => {
                                            return Err(Error::GuardFailed(e));
                                        }
                                    }
                                }
                            }
                        } else {
                            if let Some(a) = action {
                                quote! {
                                    let _data = self.context.#a(#temporary_context_call #g_a_param);
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

    // create token-streams for lifetimes
    let event_lifetimes_code = if sm.event_data.lifetimes.is_empty() {
        quote! {}
    } else {
        let event_lifetimes = &sm.event_data.all_lifetimes;
        quote! {<#(#event_lifetimes),* ,>}
    };

    let guard_failed = if let Some(ref guard_error) = sm.guard_error {
        quote! { GuardFailed(#guard_error) }
    } else {
        quote! { GuardFailed(()) }
    };

    // Build the states and events output
    quote! {
        /// This trait outlines the guards and actions that need to be implemented for the state
        /// machine.
        pub trait StateMachineContext {
            #guard_list
            #action_list
        }

        /// List of auto-generated states.
        #[allow(missing_docs)]
        #[derive(PartialEq)]
        pub enum States { #(#state_list),* }

        /// List of auto-generated events.
        #[allow(missing_docs)]
        #[derive(PartialEq)]
        pub enum Events #event_lifetimes_code { #(#event_list),* }

        /// List of possible errors
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Error {
            /// When an event is processed which should not come in the current state.
            InvalidEvent,
            /// When an event is processed whose guard did not return `true`.
            #guard_failed,
        }

        /// State machine structure definition.
        pub struct StateMachine<T: StateMachineContext> {
            state: States,
            context: T
        }

        impl<T: StateMachineContext> StateMachine<T> {
            /// Creates a new state machine with the specified starting state.
            #[inline(always)]
            pub fn new(context: T) -> Self {
                StateMachine {
                    state: States::#starting_state,
                    context
                }
            }

            /// Creates a new state machine with an initial state.
            #[inline(always)]
            pub fn new_with_state(context: T, initial_state: States) -> Self {
                StateMachine {
                    state: initial_state,
                    context
                }
            }

            /// Returns the current state.
            #[inline(always)]
            pub fn state(&self) -> &States {
                &self.state
            }

            /// Returns the current context.
            #[inline(always)]
            pub fn context(&self) -> &T {
                &self.context
            }

            /// Returns the current context as a mutable reference.
            #[inline(always)]
            pub fn context_mut(&mut self) -> &mut T {
                &mut self.context
            }

            /// Process an event.
            ///
            /// It will return `Ok(&NextState)` if the transition was successful, or `Err(Error)`
            /// if there was an error in the transition.
            pub fn process_event(&mut self, #temporary_context mut event: Events) -> Result<&States, Error> {
                match self.state {
                    #(States::#in_states => match event {
                        #(Events::#events => {
                            #code_blocks

                            Ok(&self.state)
                        }),*
                        _ => Err(Error::InvalidEvent),
                    }),*
                    _ => Err(Error::InvalidEvent),
                }
            }
        }
    }
}
