// Move guards to return a Result

use crate::parser::lifetimes::Lifetimes;
use crate::parser::ParsedStateMachine;
use proc_macro2::{Literal, Span};
use quote::quote;
use std::vec::Vec;
use syn::{punctuated::Punctuated, token::Paren, Type, TypeTuple};

pub fn generate_code(sm: &ParsedStateMachine) -> proc_macro2::TokenStream {
    // Get only the unique states
    let mut state_list: Vec<_> = sm.states.iter().map(|(_, value)| value).collect();
    state_list.sort_by_key(|state| state.to_string());

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
    event_list.sort_by_key(|event| event.to_string());

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
                        #state_name(state_data)
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
                                #value(event_data)
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
                    let state_data = match sm.state_data.data_types.get(state_name) {
                        Some(_) => quote! { state_data },
                        None => quote! {},
                    };

                    let event_data = match sm.event_data.data_types.get(name) {
                        Some(_) => quote! { event_data },
                        None => quote! {},
                    };

                    if state_data.is_empty() || event_data.is_empty() {
                        quote! { #state_data #event_data }
                    } else {
                        quote! { #state_data, #event_data }
                    }
                })
                .collect()
        })
        .collect();

    let guard_action_ref_parameters: Vec<Vec<_>> = transitions
        .iter()
        .map(|(name, value)| {
            let state_name = &sm.states.get(name).unwrap().to_string();

            value
                .iter()
                .map(|(name, _)| {
                    let state_data = match sm.state_data.data_types.get(state_name) {
                        Some(Type::Reference(_)) => quote! { state_data },
                        Some(_) => quote! { &state_data },
                        None => quote! {},
                    };

                    let event_data = match sm.event_data.data_types.get(name) {
                        Some(Type::Reference(_)) => quote! { event_data },
                        Some(_) => quote! { &event_data },
                        None => quote! {},
                    };

                    if state_data.is_empty() || event_data.is_empty() {
                        quote! { #state_data #event_data }
                    } else {
                        quote! { #state_data, #event_data }
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
        // create the state data token stream
        let state_data = match sm.state_data.data_types.get(state) {
            Some(st @ Type::Reference(_)) => quote! { state_data: #st, },
            Some(st) => quote! { state_data: &#st, },
            None => quote! {},
        };

        value.iter().for_each(|(event, value)| {
            // get input state lifetimes
            let in_state_lifetimes = sm.state_data.lifetimes.get(&value.in_state.to_string()).cloned().unwrap_or_default();

            // get output state lifetimes
            let out_state_lifetimes = sm.state_data.lifetimes.get(&value.out_state.to_string()).cloned().unwrap_or_default();

            // get event lifetimes
            let event_lifetimes = sm.event_data.lifetimes.get(event).cloned().unwrap_or_default();

            // combine all lifetimes
            let mut all_lifetimes = Lifetimes::new();
            all_lifetimes.extend(&in_state_lifetimes);
            all_lifetimes.extend(&out_state_lifetimes);
            all_lifetimes.extend(&event_lifetimes);

            // Create the guard traits for user implementation
            if let Some(guard) = &value.guard {
                let event_data = match sm.event_data.data_types.get(event) {
                    Some(et @ Type::Reference(_)) => quote! { event_data: #et },
                    Some(et) => quote! { event_data: &#et },
                    None => quote! {},
                };

                let guard_error = if sm.custom_guard_error {
                    quote! { Self::GuardError }
                } else {
                    quote! { () }
                };

                // Only add the guard if it hasn't been added before
                if !guard_set.iter().any(|g| g == guard) {
                    guard_set.push(guard.clone());
                    guard_list.extend(quote! {
                        #[allow(missing_docs)]
                        fn #guard <#all_lifetimes> (&mut self, #temporary_context #state_data #event_data) -> Result<(), #guard_error>;
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

                let state_data = match sm.state_data.data_types.get(state) {
                    Some(st) => {
                        quote! { state_data: #st, }
                    }
                    None => {
                        quote! {}
                    }
                };
                let event_data = match sm.event_data.data_types.get(event) {
                    Some(et) => {
                        quote! { event_data: #et }
                    }
                    None => {
                        quote! {}
                    }
                };

                // Only add the action if it hasn't been added before
                if !action_set.iter().any(|a| a == action) {
                    action_set.push(action.clone());
                    action_list.extend(quote! {
                        #[allow(missing_docs)]
                        fn #action <#all_lifetimes> (&mut self, #temporary_context #state_data #event_data) -> #return_type;
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
                .zip(in_states.iter().zip(out_states.iter().zip(guard_action_parameters.iter().zip(guard_action_ref_parameters.iter())))),
        )
        .map(
            |(guards, (actions, (in_state, (out_states, (guard_action_parameters, guard_action_ref_parameters)))))| {
                guards
                    .iter()
                    .zip(
                        actions
                            .iter()
                            .zip(out_states.iter().zip(guard_action_parameters.iter().zip(guard_action_ref_parameters.iter()))),
                    )
                    .map(|(guard, (action, (out_state, (g_a_param, g_a_ref_param))))| {
                        if let Some(g) = guard {
                            if let Some(a) = action {
                                quote! {
                                    if let Err(e) = self.context.#g(#temporary_context_call #g_a_ref_param) {
                                        self.state = Some(States::#in_state);
                                        return Err(Error::GuardFailed(e));
                                    }
                                    let _data = self.context.#a(#temporary_context_call #g_a_param);
                                    self.state = Some(States::#out_state);
                                }
                            } else {
                                quote! {
                                    if let Err(e) = self.context.#g(#temporary_context_call #g_a_ref_param) {
                                        self.state = Some(States::#in_state);
                                        return Err(Error::GuardFailed(e));
                                    }
                                    self.state = Some(States::#out_state);
                                }
                            }
                        } else if let Some(a) = action {
                            quote! {
                                let _data = self.context.#a(#temporary_context_call #g_a_param);
                                self.state = Some(States::#out_state);
                            }
                        } else {
                            quote! {
                                self.state = Some(States::#out_state);
                            }
                        }
                    })
                    .collect()
            },
        )
        .collect();

    let starting_state = &sm.starting_state;

    // create a token stream for creating a new machine.  If the starting state contains data, then
    // add a second argument to pass this initial data
    let starting_state_name = starting_state.to_string();
    let new_sm_code = match sm.state_data.data_types.get(&starting_state_name) {
        Some(st) => quote! {
            pub const fn new(context: T, state_data: #st ) -> Self {
                StateMachine {
                    state: Some(States::#starting_state (state_data)),
                    context
                }
            }
        },
        None => quote! {
            pub const fn new(context: T ) -> Self {
                StateMachine {
                    state: Some(States::#starting_state),
                    context
                }
            }
        },
    };

    let state_lifetimes = &sm.state_data.all_lifetimes;
    let event_lifetimes = &sm.event_data.all_lifetimes;

    // lifetimes that exists in Events but not in States
    let event_unique_lifetimes = event_lifetimes - state_lifetimes;

    // List of values for `impl<core::fmt::Display>`
    let state_display = if sm.impl_display_states {
        let list: Vec<_> = state_list
            .iter()
            .map(|value| {
                let escaped = Literal::string(&value.to_string());
                quote! { Self::#value => write!(f, #escaped) }
            })
            .collect();
        quote! {
            /// Implement core::fmt::Display for States
            impl<#state_lifetimes> core::fmt::Display for States <#state_lifetimes> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#list),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    // List of values for `impl<core::fmt::Display>`
    let event_display = if sm.impl_display_events {
        let list: Vec<_> = event_list
            .iter()
            .map(|value| {
                let escaped = Literal::string(&value.to_string());
                quote! { Self::#value => write!(f, #escaped) }
            })
            .collect();
        quote! {
            /// Implement core::fmt::Display for Events
            impl<#event_lifetimes> core::fmt::Display for Events <#event_lifetimes> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#list),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    let guard_error = if sm.custom_guard_error {
        quote! {
            /// The error type returned by guard functions.
            type GuardError: core::fmt::Debug;
        }
    } else {
        quote! {}
    };

    let error_type = if sm.custom_guard_error {
        quote! {
            Error<<T as StateMachineContext>::GuardError>
        }
    } else {
        quote! {Error}
    };

    // Build the states and events output
    quote! {
        /// This trait outlines the guards and actions that need to be implemented for the state
        /// machine.
        pub trait StateMachineContext {
            #guard_error
            #guard_list
            #action_list
        }

        /// List of auto-generated states.
        #[allow(missing_docs)]
        pub enum States <#state_lifetimes> { #(#state_list),* }

        #state_display

        /// Manually define PartialEq for States based on variant only to address issue-#21
        impl<#state_lifetimes> PartialEq for States <#state_lifetimes> {
            fn eq(&self, other: &Self) -> bool {
                use core::mem::discriminant;
                discriminant(self) == discriminant(other)
            }
        }

        /// List of auto-generated events.
        #[allow(missing_docs)]
        pub enum Events <#event_lifetimes> { #(#event_list),* }

        #event_display

        /// Manually define PartialEq for Events based on variant only to address issue-#21
        impl<#event_lifetimes> PartialEq for Events <#event_lifetimes> {
            fn eq(&self, other: &Self) -> bool {
                use core::mem::discriminant;
                discriminant(self) == discriminant(other)
            }
        }

        /// List of possible errors
        #[derive(Debug)]
        pub enum Error<T=()> {
            /// When an event is processed which should not come in the current state.
            InvalidEvent,
            /// When an event is processed whose guard did not return `true`.
            GuardFailed(T),
            /// When the state has an unexpected value.
            ///
            /// This can happen if there is a bug in the code generated by smlang,
            /// or if a guard or action gets panicked.
            Poisoned,
        }

        /// State machine structure definition.
        pub struct StateMachine<#state_lifetimes T: StateMachineContext> {
            state: Option<States <#state_lifetimes>>,
            context: T
        }

        impl<#state_lifetimes T: StateMachineContext> StateMachine<#state_lifetimes T> {
            /// Creates a new state machine with the specified starting state.
            #[inline(always)]
            #new_sm_code

            /// Creates a new state machine with an initial state.
            #[inline(always)]
            pub const fn new_with_state(context: T, initial_state: States <#state_lifetimes>) -> Self {
                StateMachine {
                    state: Some(initial_state),
                    context
                }
            }

            /// Returns the current state.
            #[inline(always)]
            pub fn state(&self) -> Result<&States <#state_lifetimes>, #error_type> {
                self.state.as_ref().ok_or_else(|| Error::Poisoned)
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
            pub fn process_event <#event_unique_lifetimes> (
                &mut self,
                #temporary_context
                mut event: Events <#event_lifetimes>
            ) -> Result<&States <#state_lifetimes>, #error_type> {
                match self.state.take().ok_or_else(|| Error::Poisoned)? {
                    #(States::#in_states => match event {
                        #(Events::#events => {
                            #code_blocks

                            self.state()
                        }),*
                        _ => {
                            self.state = Some(States::#in_states);
                            Err(Error::InvalidEvent)
                        }
                    }),*
                    state => {
                        self.state = Some(state);
                        Err(Error::InvalidEvent)
                    }
                }
            }
        }
    }
}
