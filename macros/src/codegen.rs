// Move guards to return a Result

use crate::parser::event::Transition;
use crate::parser::transition::{visit_guards, GuardExpression};
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
        .keys()
        .map(|name| in_state_pattern(sm, name))
        .collect();

    let events: Vec<Vec<_>> = transitions
        .values()
        .map(|value| {
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
        .map(|(state, value)| {
            value
                .keys()
                .map(|name| {
                    let state_data = state_data_argument(sm, state);

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
        .map(|(state, value)| {
            value
                .keys()
                .map(|name| {
                    let state_data = state_data_argument(sm, state);

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
                        .map(|transition| out_state_expression(sm, &transition.out_state))
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

    for state in transitions.keys() {
        let entry_ident = format_ident!("on_entry_{}", string_morph::to_snake_case(state));
        let state_name = format!("[{}::{}]", states_type_name, state);
        let entry_exit_async = if sm.entry_exit_async {
            quote! { async }
        } else {
            quote! {}
        };
        entries_exits.extend(quote! {
            #[doc = concat!("Called on entry to ", #state_name)]
            #[inline(always)]
            #entry_exit_async fn #entry_ident(&mut self) {}
        });
        let exit_ident = format_ident!("on_exit_{}", string_morph::to_snake_case(state));
        entries_exits.extend(quote! {
            #[doc = concat!("Called on exit from ", #state_name)]
            #[inline(always)]
            #entry_exit_async fn #exit_ident(&mut self) {}
        });
    }

    // The guards and actions of all transitions are declared in the context trait: first those
    // of the event transitions, then those of the eventless transitions, which have no event.
    for (state, event, transition) in sm.all_transitions() {
        // create the state data token stream
        let state_data = match sm.state_data.data_types.get(state) {
            Some(st @ Type::Reference(_)) => quote! { state_data: #st, },
            Some(st) => quote! { state_data: &#st, },
            None => quote! {},
        };

        // get input state lifetimes
        let in_state_lifetimes = sm
            .state_data
            .lifetimes
            .get(state)
            .cloned()
            .unwrap_or_default();

        // get output state lifetimes
        let out_state_lifetimes = sm
            .state_data
            .lifetimes
            .get(&transition.out_state.to_string())
            .cloned()
            .unwrap_or_default();

        // get event lifetimes, an eventless transition has no event
        let event_lifetimes = event
            .and_then(|event| sm.event_data.lifetimes.get(event))
            .cloned()
            .unwrap_or_default();

        // combine all lifetimes
        let mut all_lifetimes = Lifetimes::new();
        all_lifetimes.extend(&in_state_lifetimes);
        all_lifetimes.extend(&out_state_lifetimes);
        all_lifetimes.extend(&event_lifetimes);

        // get the event data type, an eventless transition has no event data
        let event_data_type = event.and_then(|event| sm.event_data.data_types.get(event));

        // Create the guard traits for user implementation
        if let Some(guard_expression) = &transition.guard {
            visit_guards(guard_expression,|guard| {
                let is_async = guard.is_async;
                let guard = &guard.ident;
                let event_data = match event_data_type {
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

            let event_data = match event_data_type {
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

    let temporary_context_call = match &sm.temporary_context_type {
        Some(_) => {
            quote! { temporary_context, }
        }
        None => {
            quote! {}
        }
    };

    let error_type = if sm.custom_error {
        quote! {
            #error_type_name<<T as #state_machine_context_type_name>::Error>
        }
    } else {
        quote! {#error_type_name}
    };

    let state_lifetimes = &sm.state_data.all_lifetimes;

    // What `process_event` and `process_eventless` return: the new state, or an error.
    let process_result = quote! {
        Result<&#states_type_name <#state_lifetimes>, #error_type>
    };

    // `process_eventless` is async if it awaits the entry and exit hooks, or any guard or action
    // of the eventless transitions.
    let eventless_is_async = sm.entry_exit_async
        || sm
            .eventless_transitions
            .values()
            .flatten()
            .any(is_async_transition);

    // `process_event` is async if anything it calls is async, which includes `process_eventless`.
    let is_async_state_machine = sm.entry_exit_async
        || sm
            .all_transitions()
            .any(|(_, _, transition)| is_async_transition(transition));

    let entry_exit_await = if sm.entry_exit_async {
        quote! { .await }
    } else {
        quote! {}
    };

    let codegen = TransitionCodegen {
        states_type_name: &states_type_name,
        error_type_name: &error_type_name,
        temporary_context_call: &temporary_context_call,
        entry_exit_await: &entry_exit_await,
    };

    // The temporary context is only passed to `process_eventless` if one of its guards or actions
    // takes it. Otherwise an action of the event transition may have consumed it already.
    let eventless_uses_context = sm
        .eventless_transitions
        .values()
        .flatten()
        .any(|transition| transition.guard.is_some() || transition.action.is_some());
    let (eventless_temporary_context, eventless_temporary_context_call) = if eventless_uses_context
    {
        (temporary_context.clone(), temporary_context_call.clone())
    } else {
        (quote! {}, quote! {})
    };

    // The method that takes the eventless transitions, empty without eventless transitions.
    let eventless_method = generate_eventless_method(
        sm,
        &codegen,
        eventless_is_async,
        &eventless_temporary_context,
        &process_result,
    );
    let eventless_await = if eventless_is_async {
        quote! { .await }
    } else {
        quote! {}
    };

    // With eventless transitions, the returned state is the state after all of them.
    let eventless_doc = if sm.eventless_transitions.is_empty() {
        quote! {}
    } else {
        quote! {
            ///
            /// After a transition, the eventless transitions of the resulting state are taken, until
            /// the machine reaches a state without an enabled eventless transition. So the returned
            /// state is the state after all eventless transitions have been taken. If a guard or
            /// an action of an eventless transition returns an error, nothing is rolled back and the
            /// machine stays in the source state of that eventless transition. Call
            /// `process_eventless` to retry it.
        }
    };

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

                                // An event transition ends by returning the new state. If the
                                // new state has eventless transitions, they are taken first, and
                                // the state they end in is returned. Without eventless
                                // transitions, no extra code is generated.
                                let tail = if sm.eventless_transitions.contains_key(out_state_string) {
                                    quote!{
                                        return self.process_eventless(#eventless_temporary_context_call) #eventless_await;
                                    }
                                } else {
                                    quote!{
                                        return Ok(&self.state);
                                    }
                                };

                                let transition = codegen.transition(
                                    in_state_string,
                                    out_state_string,
                                    out_state,
                                    action,
                                    action_params,
                                    tail,
                                );

                                codegen.guarded(guard, guard_params, transition)
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

    let states_attr_list = &sm.states_attr;
    let events_attr_list = &sm.events_attr;
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
            /// `process_event()`. No-op by default which can be overridden in implementations
            /// of a state machine's `StateMachineContext` trait.
            fn transition_callback(&self, old_state: & #states_type_name, new_state: & #states_type_name) {}
        }

        /// List of auto-generated states.
        #[allow(missing_docs)]
        #(#states_attr_list)*
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
        #(#events_attr_list)*
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
            #eventless_doc
            pub #is_async fn process_event <#event_unique_lifetimes> (
                &mut self,
                #temporary_context
                event: #events_type_name <#event_lifetimes>
            ) -> #process_result {
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

            #eventless_method
        }
    }
}

/// Whether the guard or the action of a transition is async.
fn is_async_transition(transition: &Transition) -> bool {
    let mut is_async = transition
        .action
        .as_ref()
        .is_some_and(|action| action.is_async);
    if let Some(guard_expression) = &transition.guard {
        visit_guards(guard_expression, |guard| {
            is_async |= guard.is_async;
            Ok(())
        })
        .unwrap();
    }
    is_async
}

/// The pattern that matches a state and binds its data, if it has any: `X` or
/// `X(ref state_data)`.
fn in_state_pattern(sm: &ParsedStateMachine, state: &str) -> TokenStream {
    let state_ident = &sm.states[state];
    match sm.state_data.data_types.get(state) {
        None => quote! { #state_ident },
        Some(_) => quote! { #state_ident(ref state_data) },
    }
}

/// The expression that creates a new state: `X` or `X(_data)` for a state with data, which the
/// action of the transition provides as `_data`.
fn out_state_expression(sm: &ParsedStateMachine, out_state: &Ident) -> TokenStream {
    match sm.state_data.data_types.get(&out_state.to_string()) {
        None => quote! { #out_state },
        Some(_) => quote! { #out_state(_data) },
    }
}

/// The argument that passes the data of the source state, if it has any, to a guard or an
/// action.
fn state_data_argument(sm: &ParsedStateMachine, state: &str) -> TokenStream {
    match sm.state_data.data_types.get(state) {
        Some(Type::Reference(_)) => quote! { state_data },
        Some(_) => quote! { &state_data },
        None => quote! {},
    }
}

fn generate_action(
    action: &Option<AsyncIdent>,
    temporary_context_call: &TokenStream,
    g_a_param: &TokenStream,
    error_type_name: &Ident,
) -> TokenStream {
    if let Some(AsyncIdent {
        ident: action_ident,
        is_async: is_a_async,
    }) = action
    {
        let action_await = if *is_a_async {
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
    }
}

/// The parts of the generated code that all transitions have in common, event transitions and
/// eventless transitions alike.
struct TransitionCodegen<'a> {
    states_type_name: &'a Ident,
    error_type_name: &'a Ident,
    temporary_context_call: &'a TokenStream,
    entry_exit_await: &'a TokenStream,
}

impl TransitionCodegen<'_> {
    /// Generates the code that takes a transition: `on_exit` of the source state, the action,
    /// the transition callback, the assignment of the new state, and `on_entry` of the target
    /// state. The code ends with `tail`, which decides what happens next. A transition within
    /// the same state skips `on_exit`, the callback and `on_entry`.
    ///
    /// `out_state` is the new state as an expression, including its data.
    fn transition(
        &self,
        in_state_name: &str,
        out_state_name: &str,
        out_state: &TokenStream,
        action: &Option<AsyncIdent>,
        action_params: &TokenStream,
        tail: TokenStream,
    ) -> TokenStream {
        let states_type_name = self.states_type_name;
        let entry_exit_await = self.entry_exit_await;
        let entry_ident = format_ident!("on_entry_{}", string_morph::to_snake_case(out_state_name));
        let exit_ident = format_ident!("on_exit_{}", string_morph::to_snake_case(in_state_name));

        let action_code = generate_action(
            action,
            self.temporary_context_call,
            action_params,
            self.error_type_name,
        );

        if in_state_name == out_state_name {
            // Stay in the same state => no need to call on_entry/on_exit
            quote! {
                #action_code
                self.state = #states_type_name::#out_state;
                #tail
            }
        } else {
            quote! {
                self.context.#exit_ident()#entry_exit_await;
                #action_code
                let out_state = #states_type_name::#out_state;
                self.context().transition_callback(&self.state, &out_state);
                self.state = out_state;
                self.context.#entry_ident()#entry_exit_await;
                #tail
            }
        }
    }

    /// Wraps the code of a transition (see [`Self::transition`]) so that it only runs if the
    /// guard passes. A transition without a guard is returned as it is.
    fn guarded(
        &self,
        guard: &Option<GuardExpression>,
        guard_params: &TokenStream,
        transition: TokenStream,
    ) -> TokenStream {
        let Some(expr) = guard else {
            // Unguarded transition
            return transition;
        };

        // Guarded transition
        let temporary_context_call = self.temporary_context_call;
        let error_type_name = self.error_type_name;
        let guard_expression = expr.to_token_stream(&mut |async_ident: &AsyncIdent| {
            let guard_ident = &async_ident.ident;
            let guard_await = if async_ident.is_async {
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
                #transition
            }
        }
    }
}

/// Generates `process_eventless`, which `process_event` calls after a transition into a state
/// that has eventless transitions, and which is also part of the public API. Without eventless
/// transitions it returns nothing.
///
/// The method takes the eventless transitions of the current state in declaration order: the
/// first one whose guard passes, or that has no guard, fires, and the same is repeated for the
/// new state. It stops, and returns the state, in a state where no eventless transition is
/// enabled. This is a loop, not a recursion: an async recursion would need boxing, which
/// `no_std` without `alloc` cannot do. Guards can end the loop, so it must not assume that it
/// terminates.
fn generate_eventless_method(
    sm: &ParsedStateMachine,
    codegen: &TransitionCodegen,
    is_async: bool,
    temporary_context: &TokenStream,
    result: &TokenStream,
) -> TokenStream {
    if sm.eventless_transitions.is_empty() {
        return TokenStream::new();
    }

    let states_type_name = codegen.states_type_name;

    let arms: Vec<TokenStream> = sm
        .eventless_transitions
        .iter()
        .map(|(source, transitions)| {
            let pattern = in_state_pattern(sm, source);
            let params = state_data_argument(sm, source);

            let steps: Vec<TokenStream> = transitions
                .iter()
                .map(|transition| {
                    let out_state_name = transition.out_state.to_string();
                    let out_state_tokens = out_state_expression(sm, &transition.out_state);

                    // Look at the eventless transitions of the new state, if it has any.
                    // Otherwise the machine rests there.
                    let tail = if sm.eventless_transitions.contains_key(&out_state_name) {
                        quote! { continue; }
                    } else {
                        quote! { return Ok(&self.state); }
                    };

                    let step = codegen.transition(
                        source,
                        &out_state_name,
                        &out_state_tokens,
                        &transition.action,
                        &params,
                        tail,
                    );
                    codegen.guarded(&transition.guard, &params, step)
                })
                .collect();

            quote! {
                #states_type_name::#pattern => {
                    #(#steps)*
                }
            }
        })
        .collect();

    let is_async_fn = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    // A transition that fires ends in `continue` or `return`. If none fires, the machine rests
    // in its state, which is not an error.
    quote! {
        /// Takes the eventless transitions of the current state, until the machine reaches a state
        /// without an enabled eventless transition, and returns that state.
        ///
        /// `process_event` already does this after each transition. Call it yourself to:
        /// - retry after an eventless guard or action returned an error. The machine stays in the
        ///   source state of the failed transition, and this method tries it again, the same way
        ///   sending the event again retries an event transition. As there, `on_exit` of that state
        ///   runs again when its action is retried.
        /// - settle a machine created with `new_with_state` in a state that has eventless transitions.
        /// - check the guards of a resting state again, for example to poll a condition
        ///   (`Waiting [ready] = Go`) without an event.
        ///
        /// If no eventless transition of the current state is enabled, nothing happens and the
        /// current state is returned.
        #[allow(unused_variables, clippy::never_loop)]
        pub #is_async_fn fn process_eventless(&mut self, #temporary_context) -> #result {
            loop {
                match self.state {
                    #(#arms)*
                    #[allow(unreachable_patterns)]
                    _ => {}
                }
                return Ok(&self.state);
            }
        }
    }
}
