// Move guards to return a Result

use crate::parser::transition::visit_guards;
use crate::parser::{lifetimes::Lifetimes, AsyncIdent, ParsedStateMachine};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::Type;

pub fn generate_code(sm: &ParsedStateMachine) -> proc_macro2::TokenStream {
    let (sm_name, sm_name_span) = sm
        .name
        .as_ref()
        .map(|name| (name.to_string(), name.span()))
        .unwrap_or_else(|| (String::new(), Span::call_site()));
    let states_type_name = format_ident!("{sm_name}States", span = sm_name_span);
    let events_type_name = format_ident!("{sm_name}Events", span = sm_name_span);
    let error_type_name = format_ident!("{sm_name}Error", span = sm_name_span);
    let state_machine_type_name = format_ident!("{sm_name}StateMachine", span = sm_name_span);
    let state_machine_context_type_name =
        format_ident!("{sm_name}StateMachineContext", span = sm_name_span);

    // Get only the unique states
    let mut state_list: Vec<_> = sm.states.values().collect();
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
    let mut event_list: Vec<_> = sm.events.values().collect();
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
                                #value(event_data)
                            }
                        }
                    }
                })
                .collect()
        })
        .collect();

    // Map guards, actions and output states into code blocks
    let guards: Vec<Vec<_>> = transitions
        .values()
        .map(|event_mappings| {
            event_mappings
                .values()
                .map(|event_mapping| {
                    event_mapping
                        .transitions
                        .iter()
                        .map(|transition| transition.guard.clone())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let actions: Vec<Vec<_>> = transitions
        .values()
        .map(|event_mappings| {
            event_mappings
                .values()
                .map(|event_mapping| {
                    event_mapping
                        .transitions
                        .iter()
                        .map(|transition| transition.action.clone())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let action_parameters: Vec<Vec<_>> = transitions
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

    let guard_parameters: Vec<Vec<_>> = transitions
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

    let custom_error = if sm.custom_error {
        quote! { Self::Error }
    } else {
        quote! { () }
    };

    let out_states: Vec<Vec<Vec<TokenStream>>> = transitions
        .values()
        .map(|event_mappings| {
            event_mappings
                .values()
                .map(|event_mapping| {
                    event_mapping
                        .transitions
                        .iter()
                        .map(|transition| transition.out_state.clone())
                        .map(|out_state| {
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
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
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

    let mut entries_exits = proc_macro2::TokenStream::new();

    for (state, event_mappings) in transitions.iter() {
        // create the state data token stream
        let state_data = match sm.state_data.data_types.get(state) {
            Some(st @ Type::Reference(_)) => quote! { state_data: #st, },
            Some(st) => quote! { state_data: &#st, },
            None => quote! {},
        };

        let entry_ident = format_ident!("on_entry_{}", string_morph::to_snake_case(state));
        let state_name = format!("[{}::{}]", states_type_name, state);
        entries_exits.extend(quote! {
            #[doc = concat!("Called on entry to ", #state_name)]
            #[inline(always)]
            fn #entry_ident(&mut self) {}
        });
        let exit_ident = format_ident!("on_exit_{}", string_morph::to_snake_case(state));
        entries_exits.extend(quote! {
            #[doc = concat!("Called on exit from ", #state_name)]
            #[inline(always)]
            fn #exit_ident(&mut self) {}
        });

        for (event, event_mapping) in event_mappings {
            for transition in &event_mapping.transitions {
                // get input state lifetimes
                let in_state_lifetimes = sm
                    .state_data
                    .lifetimes
                    .get(&event_mapping.in_state.to_string())
                    .cloned()
                    .unwrap_or_default();

                // get output state lifetimes
                let out_state_lifetimes = sm
                    .state_data
                    .lifetimes
                    .get(&transition.out_state.to_string())
                    .cloned()
                    .unwrap_or_default();

                // get event lifetimes
                let event_lifetimes = sm
                    .event_data
                    .lifetimes
                    .get(event)
                    .cloned()
                    .unwrap_or_default();

                // combine all lifetimes
                let mut all_lifetimes = Lifetimes::new();
                all_lifetimes.extend(&in_state_lifetimes);
                all_lifetimes.extend(&out_state_lifetimes);
                all_lifetimes.extend(&event_lifetimes);

                // Create the guard traits for user implementation
                if let Some(guard_expression) = &transition.guard {
                    visit_guards(guard_expression,|guard| {
                        let is_async = guard.is_async;
                        let guard = &guard.ident;
                        let event_data = match sm.event_data.data_types.get(event) {
                            Some(et @ Type::Reference(_)) => quote! { event_data: #et },
                            Some(et) => quote! { event_data: &#et },
                            None => quote! {},
                        };

                        // Only add the guard if it hasn't been added before
                        if !guard_set.iter().any(|g| g == guard) {
                            guard_set.push(guard.clone());
                            let is_async = if is_async { quote!{ async } } else { quote!{ } };
                            guard_list.extend(quote! {
                            #[allow(missing_docs)]
                            #[allow(clippy::result_unit_err)]
                            #is_async fn #guard <#all_lifetimes> (&self, #temporary_context #state_data #event_data) -> Result<bool,#custom_error>;
                        });
                        };
                        Ok(())
                    }).unwrap();
                }

                // Create the action traits for user implementation
                if let Some(AsyncIdent {
                    ident: action,
                    is_async,
                }) = &transition.action
                {
                    let is_async = if *is_async {
                        quote! { async }
                    } else {
                        quote! {}
                    };
                    let return_type = if let Some(output_data) = sm
                        .state_data
                        .data_types
                        .get(&transition.out_state.to_string())
                    {
                        quote! { Result<#output_data,#custom_error> }
                    } else {
                        // Empty return type
                        quote! { Result<(),#custom_error> }
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
                            #[allow(clippy::unused_unit)]
                            #is_async fn #action <#all_lifetimes> (&mut self, #temporary_context #state_data #event_data) -> #return_type;
                        });
                    }
                }
            }
        }
    }

    let temporary_context_call = match &sm.temporary_context_type {
        Some(_) => {
            quote! { temporary_context, }
        }
        None => {
            quote! {}
        }
    };

    let mut is_async_state_machine = false;

    // Create the code blocks inside the switch cases
    let code_blocks: Vec<Vec<_>> = guards
        .iter()
        .zip(
            actions
                .iter()
                .zip(in_states.iter().zip(out_states.iter().zip(action_parameters.iter().zip(guard_parameters.iter())))),
        )
        .map(
            |(guards, (actions, (in_state, (out_states, (action_parameters, guard_parameters)))))| {
                guards
                    .iter()
                    .zip(
                        actions
                            .iter()
                            .zip(out_states.iter().zip(action_parameters.iter().zip(guard_parameters.iter()))),
                    )
                    .map(|(guard, (action, (out_state, (action_params, guard_params))))| {
                        let streams: Vec<TokenStream> =
                            guard.iter()
                            .zip(action.iter().zip(out_state)).map(|(guard, (action,out_state))| {
                                let binding = out_state.to_string();
                                let out_state_string = binding.split('(').next().unwrap().trim();
                                let binding = in_state.to_string();
                                let in_state_string = binding.split('(').next().unwrap().trim();

                                let entry_ident = format_ident!("on_entry_{}", string_morph::to_snake_case(out_state_string));
                                let exit_ident = format_ident!("on_exit_{}", string_morph::to_snake_case(in_state_string));

                                let entry_exit_states =
                                    quote! {
                                        self.context.#exit_ident();
                                        self.context.#entry_ident();
                                        };
                                let (is_async_action, action_code) = generate_action(action, &temporary_context_call, action_params, &error_type_name);
                                is_async_state_machine |= is_async_action;

                                let transition = if in_state_string == out_state_string {
                                    // Stay in the same state => no need to call on_entry/on_exit and log_state_change
                                    quote!{
                                            self.state = #states_type_name::#out_state;
                                            return Ok(&self.state);
                                        }

                                } else {
                                    quote!{
                                            let out_state = #states_type_name::#out_state;
                                            self.context.log_state_change(&out_state);
                                            #entry_exit_states
                                            self.context().transition_callback(&self.state, &out_state);
                                            self.state = out_state;
                                            return Ok(&self.state);
                                        }
                                };

                                if let Some(expr) = guard { // Guarded transition
                                    let guard_expression= expr.to_token_stream(&mut |async_ident: &AsyncIdent| {
                                        let guard_ident = &async_ident.ident;
                                        let guard_await = if async_ident.is_async {
                                            is_async_state_machine = true;
                                            quote! { .await }
                                        } else {
                                            quote! {}
                                        };
                                        quote! {
                                            self.context.#guard_ident(#temporary_context_call #guard_params) #guard_await .map_err(#error_type_name::GuardFailed)?
                                        }
                                    });
                                    quote! {
                                        // This #guard_expression contains a boolean expression of guard functions
                                        // Each guard function has Result<bool,_> return type.
                                        // For example, [ f && !g ] will expand into
                                        //  self.context.f()? && !self.context.g()?
                                        let guard_passed = #guard_expression;
                                        self.context.log_guard(stringify!(#guard_expression), guard_passed);

                                        // If the guard passed, we transition immediately.
                                        // Otherwise, there may be a later transition that passes,
                                        // so we'll defer to that.
                                        if guard_passed {
                                            #action_code
                                            #transition
                                        }
                                    }
                                } else { // Unguarded transition
                                   quote!{
                                        #action_code
                                        #transition
                                   }
                                }
                            }
                            ).collect();
                        quote!{
                            #(#streams)*
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
                #state_machine_type_name {
                    state: #states_type_name::#starting_state (state_data),
                    context
                }
            }
        },
        None => quote! {
            pub const fn new(context: T ) -> Self {
                #state_machine_type_name {
                    state: #states_type_name::#starting_state,
                    context
                }
            }
        },
    };

    let state_lifetimes = &sm.state_data.all_lifetimes;
    let event_lifetimes = &sm.event_data.all_lifetimes;

    // lifetimes that exists in #events_type_name but not in #states_type_name
    let event_unique_lifetimes = event_lifetimes - state_lifetimes;

    let custom_error = if sm.custom_error {
        quote! {
            /// The error type returned by guard or action functions.
            type Error: core::fmt::Debug;
        }
    } else {
        quote! {}
    };

    let is_async = if is_async_state_machine {
        quote! { async }
    } else {
        quote! {}
    };

    let error_type = if sm.custom_error {
        quote! {
            #error_type_name<<T as #state_machine_context_type_name>::Error>
        }
    } else {
        quote! {#error_type_name}
    };

    let derive_states_list = &sm.derive_states;
    let derive_events_list = &sm.derive_events;
    // Build the states and events output
    quote! {
        /// This trait outlines the guards and actions that need to be implemented for the state
        /// machine.
        pub trait #state_machine_context_type_name {
            #custom_error
            #guard_list
            #action_list
            #entries_exits


            /// Called at the beginning of a state machine's `process_event()`. No-op by
            /// default but can be overridden in implementations of a state machine's
            /// `StateMachineContext` trait.
            fn log_process_event(&self, current_state: & #states_type_name, event: & #events_type_name) {}

            /// Called after executing a guard during `process_event()`. No-op by
            /// default but can be overridden in implementations of a state machine's
            /// `StateMachineContext` trait.
            fn log_guard(&self, guard: &'static str, result: bool) {}

            /// Called after executing an action during `process_event()`. No-op by
            /// default but can be overridden in implementations of a state machine's
            /// `StateMachineContext` trait.
            fn log_action(&self, action: &'static str) {}

            /// Called when transitioning to a new state as a result of an event passed to
            /// `process_event()`. No-op by default but can be overridden in implementations
            /// of a state machine's `StateMachineContext` trait.
            fn log_state_change(&self, new_state: & #states_type_name) {}

            /// Called when transitioning to a new state as a result of an event passed to
            /// `process_event()`. No-op by default which can be overridden in implementations
            /// of a state machine's `StateMachineContext` trait.
            fn transition_callback(&self, old_state: & #states_type_name, new_state: & #states_type_name) {}
        }

        /// List of auto-generated states.
        #[allow(missing_docs)]
        #[derive(#(#derive_states_list),*)]
        pub enum #states_type_name <#state_lifetimes> { #(#state_list),* }

        /// Manually define PartialEq for #states_type_name based on variant only to address issue-#21
        impl<#state_lifetimes> PartialEq for #states_type_name <#state_lifetimes> {
            fn eq(&self, other: &Self) -> bool {
                use core::mem::discriminant;
                discriminant(self) == discriminant(other)
            }
        }

        /// List of auto-generated events.
        #[allow(missing_docs)]
        #[derive(#(#derive_events_list),*)]
        pub enum #events_type_name <#event_lifetimes> { #(#event_list),* }

        /// Manually define PartialEq for #events_type_name based on variant only to address issue-#21
        impl<#event_lifetimes> PartialEq for #events_type_name <#event_lifetimes> {
            fn eq(&self, other: &Self) -> bool {
                use core::mem::discriminant;
                discriminant(self) == discriminant(other)
            }
        }

        /// List of possible errors
        #[derive(Debug,PartialEq)]
        pub enum #error_type_name  <T=()> {
            /// When an event is processed which should not come in the current state.
            InvalidEvent,
            /// When an event is processed and none of the transitions happened.
            TransitionsFailed,
            /// When guard is failed.
            GuardFailed(T),
            /// When action returns Err
            ActionFailed(T),
        }

        /// State machine structure definition.
        pub struct #state_machine_type_name<#state_lifetimes T: #state_machine_context_type_name> {
            state: #states_type_name <#state_lifetimes>,
            context: T
        }

        impl<#state_lifetimes T: #state_machine_context_type_name> #state_machine_type_name<#state_lifetimes T> {
            /// Creates a new state machine with the specified starting state.
            #[inline(always)]
            #new_sm_code

            /// Creates a new state machine with an initial state.
            #[inline(always)]
            pub const fn new_with_state(context: T, initial_state: #states_type_name <#state_lifetimes>) -> Self {
                #state_machine_type_name {
                    state: initial_state,
                    context
                }
            }

            /// Returns the current state.
            #[inline(always)]
            pub fn state(&self) -> &#states_type_name <#state_lifetimes> {
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
            /// It will return `Ok(&NextState)` if the transition was successful, or `Err(#error_type_name)`
            /// if there was an error in the transition.
            pub #is_async fn process_event <#event_unique_lifetimes> (
                &mut self,
                #temporary_context
                event: #events_type_name <#event_lifetimes>
            ) -> Result<&#states_type_name <#state_lifetimes>, #error_type> {
                self.context.log_process_event(self.state(), &event);
               match self.state {
                    #(
                    #[allow(clippy::match_single_binding)]
                    #states_type_name::#in_states => match event {
                        #(#events_type_name::#events => {
                            #code_blocks

                            #[allow(unreachable_code)]
                            {
                                // none of the guarded or non-guarded transitions occurred,
                                Err(#error_type_name ::TransitionsFailed)
                            }
                        }),*
                        #[allow(unreachable_patterns)]
                        _ => Err(#error_type_name ::InvalidEvent),
                    }),*
                }
            }
        }
    }
}
fn generate_action(
    action: &Option<AsyncIdent>,
    temporary_context_call: &TokenStream,
    g_a_param: &TokenStream,
    error_type_name: &Ident,
) -> (bool, TokenStream) {
    let mut is_async = false;
    let code = if let Some(AsyncIdent {
        ident: action_ident,
        is_async: is_a_async,
    }) = action
    {
        let action_await = if *is_a_async {
            is_async = true;
            quote! { .await }
        } else {
            quote! {}
        };
        quote! {
            // ACTION
            let _data = self.context.#action_ident(#temporary_context_call #g_a_param) #action_await .map_err(#error_type_name::ActionFailed)?;
            self.context.log_action(stringify!(#action_ident));
        }
    } else {
        quote! {}
    };
    (is_async, code)
}
