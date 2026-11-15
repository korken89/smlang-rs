pub mod data;
pub mod event;
pub mod input_state;
pub mod lifetimes;
pub mod output_state;
pub mod state_machine;
pub mod transition;

use data::DataDefinitions;
use event::{Event, EventMapping};
use state_machine::StateMachine;

use input_state::InputState;
use proc_macro2::{Span, TokenStream};

use crate::parser::event::Transition;
use std::collections::{hash_map, HashMap};
use std::fmt;
use syn::{parse, Attribute, Ident, Type};
use transition::StateTransition;
pub type TransitionMap = HashMap<String, HashMap<String, EventMapping>>;
/// Eventless transitions by source state name, in declaration order.
pub type EventlessMap = HashMap<String, Vec<Transition>>;

#[derive(Debug, Clone)]
pub struct AsyncIdent {
    pub ident: Ident,
    pub is_async: bool,
}
impl AsyncIdent {
    pub fn to_token_stream<F>(&self, visit: &mut F) -> TokenStream
    where
        F: FnMut(&AsyncIdent) -> TokenStream,
    {
        visit(self)
    }
}
impl fmt::Display for AsyncIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_async {
            write!(f, "{}().await", self.ident)
        } else {
            write!(f, "{}()", self.ident)
        }
    }
}

#[derive(Debug)]
pub struct ParsedStateMachine {
    pub name: Option<Ident>,
    pub states_attr: Vec<Attribute>,
    pub events_attr: Vec<Attribute>,
    pub temporary_context_type: Option<Type>,
    pub custom_error: bool,
    pub states: HashMap<String, Ident>,
    pub starting_state: Ident,
    pub state_data: DataDefinitions,
    pub events: HashMap<String, Ident>,
    pub event_data: DataDefinitions,
    pub states_events_mapping: HashMap<String, HashMap<String, EventMapping>>,
    /// Eventless transitions by source state name, in declaration order. Only states that
    /// have eventless transitions are present. These transitions are not part of
    /// `states_events_mapping`, `events` or `event_data`.
    pub eventless_transitions: EventlessMap,
    pub entry_exit_async: bool,
}

// Checks that a transition into a state with data has an action that provides the data.
// Shared by event transitions and eventless transitions.
fn check_out_state_data_has_action(
    transition: &StateTransition,
    state_data: &DataDefinitions,
) -> Result<(), parse::Error> {
    if state_data
        .data_types
        .contains_key(&transition.out_state.ident.to_string())
        && transition.action.is_none()
    {
        // This transition goes to a state that has data associated, but it has no action to
        // provide the data.
        return Err(parse::Error::new(
            transition.out_state.ident.span(),
            "This state has data associated, but not action is define here to provide it.",
        ));
    }
    Ok(())
}

// helper function for adding an event transition to a transition event map
fn add_transition(
    transition: &StateTransition,
    event: &Event,
    transition_map: &mut TransitionMap,
    state_data: &DataDefinitions,
) -> Result<(), parse::Error> {
    let p = transition_map
        .get_mut(&transition.in_state.ident.to_string())
        .unwrap();

    match p.entry(event.ident.to_string()) {
        hash_map::Entry::Vacant(entry) => {
            let mapping = EventMapping {
                in_state: transition.in_state.ident.clone(),
                event: event.ident.clone(),
                transitions: vec![Transition {
                    guard: transition.guard.clone(),
                    action: transition.action.clone(),
                    out_state: transition.out_state.ident.clone(),
                }],
            };
            entry.insert(mapping);
        }
        hash_map::Entry::Occupied(mut entry) => {
            let mapping = entry.get_mut();
            mapping.transitions.push(Transition {
                guard: transition.guard.clone(),
                action: transition.action.clone(),
                out_state: transition.out_state.ident.clone(),
            });
        }
    }

    // Check for actions when states have data
    check_out_state_data_has_action(transition, state_data)
}

impl ParsedStateMachine {
    pub fn new(mut sm: StateMachine) -> parse::Result<Self> {
        // Derive out_state for internal non-wildcard transitions
        for transition in sm.transitions.iter_mut() {
            if transition.out_state.internal_transition && !transition.in_state.wildcard {
                transition.out_state.ident = transition.in_state.ident.clone();
                transition
                    .out_state
                    .data_type
                    .clone_from(&transition.in_state.data_type);
                transition.out_state.internal_transition = false;
            }
        }

        // Check the initial state definition
        let mut starting_transitions_iter = sm.transitions.iter().filter(|sm| sm.in_state.start);

        let starting_transition = starting_transitions_iter.next().ok_or(parse::Error::new(
            Span::call_site(),
            "No starting state defined, indicate the starting state with a *.",
        ))?;

        if starting_transitions_iter.next().is_some() {
            return Err(parse::Error::new(
                Span::call_site(),
                "More than one starting state defined (indicated with *), remove duplicates.",
            ));
        }

        // Extract the starting state
        let starting_state = starting_transition.in_state.ident.clone();

        // The starting state cannot have eventless transitions, because `new()` is a const fn
        // that cannot run guards or actions. This is checked by state name, so that
        // `*S1 + E = S2, S1 [g] = S3` is also rejected.
        if let Some(eventless) = sm
            .transitions
            .iter()
            .find(|t| t.event.is_none() && t.in_state.ident == starting_state)
        {
            return Err(parse::Error::new(
                eventless.in_state.ident.span(),
                "The starting state cannot have eventless transitions, because `new()` cannot run guards or actions",
            ));
        }

        let mut states = HashMap::new();
        let mut state_data = DataDefinitions::new();
        let mut events = HashMap::new();
        let mut event_data = DataDefinitions::new();
        let mut states_events_mapping = TransitionMap::new();
        let mut eventless_transitions = EventlessMap::new();

        for transition in sm.transitions.iter() {
            // Collect states
            let in_state_name = transition.in_state.ident.to_string();
            if !transition.in_state.wildcard {
                states.insert(in_state_name.clone(), transition.in_state.ident.clone());
                state_data.collect(in_state_name.clone(), transition.in_state.data_type.clone())?;
            }
            if !transition.out_state.internal_transition {
                let out_state_name = transition.out_state.ident.to_string();
                states.insert(out_state_name.clone(), transition.out_state.ident.clone());
                state_data.collect(
                    out_state_name.clone(),
                    transition.out_state.data_type.clone(),
                )?;
            }

            // Collect events. Eventless transitions have none.
            if let Some(event) = &transition.event {
                let event_name = event.ident.to_string();
                events.insert(event_name.clone(), event.ident.clone());
                event_data.collect(event_name.clone(), event.data_type.clone())?;
            }

            // add input and output states to the mapping HashMap. This includes states that
            // only appear in eventless transitions, so they still get entry/exit hooks and
            // `process_event` match arms.
            if !transition.in_state.wildcard {
                states_events_mapping.insert(transition.in_state.ident.to_string(), HashMap::new());
            }
            if !transition.out_state.internal_transition {
                states_events_mapping
                    .insert(transition.out_state.ident.to_string(), HashMap::new());
            }
        }

        for transition in sm.transitions.iter() {
            // Eventless transitions are kept apart from the event transitions. They never
            // take part in wildcard expansion (a wildcard source is rejected by the parser).
            let event = match &transition.event {
                Some(event) => event,
                None => {
                    check_out_state_data_has_action(transition, &state_data)?;
                    eventless_transitions
                        .entry(transition.in_state.ident.to_string())
                        .or_default()
                        .push(Transition {
                            guard: transition.guard.clone(),
                            action: transition.action.clone(),
                            out_state: transition.out_state.ident.clone(),
                        });
                    continue;
                }
            };

            // if input state is a wildcard, we need to add this transition for all states
            if transition.in_state.wildcard {
                let mut transition_added = false;

                for (name, in_state) in &states {
                    // skip already set input state
                    let p = states_events_mapping
                        .get_mut(&in_state.to_string())
                        .unwrap();

                    if p.contains_key(&event.ident.to_string()) {
                        continue;
                    }

                    // create a new input state from wildcard
                    let in_state = InputState {
                        start: false,
                        wildcard: false,
                        ident: in_state.clone(),
                        data_type: state_data.data_types.get(name).cloned(),
                    };

                    // create the transition
                    let mut out_state = transition.out_state.clone();
                    if out_state.internal_transition {
                        out_state.ident = in_state.ident.clone();
                        out_state.data_type.clone_from(&in_state.data_type);
                    }
                    let wildcard_transition = StateTransition {
                        in_state,
                        event: transition.event.clone(),
                        guard: transition.guard.clone(),
                        action: transition.action.clone(),
                        out_state,
                    };

                    // add the wildcard transition to the transition map
                    // TODO:  Need to work on the span of this error, as it is being caused by the wildcard
                    // but won't show up at that line
                    add_transition(
                        &wildcard_transition,
                        event,
                        &mut states_events_mapping,
                        &state_data,
                    )?;

                    transition_added = true;
                }

                // No transitions were added by expanding the wildcard,
                // so emit an error to the user
                if !transition_added {
                    return Err(parse::Error::new(
                        transition.in_state.ident.span(),
                        "Wildcard has no effect",
                    ));
                }
            } else {
                add_transition(transition, event, &mut states_events_mapping, &state_data)?;
            }
        }

        Ok(ParsedStateMachine {
            name: sm.name,
            states_attr: sm.states_attr,
            events_attr: sm.events_attr,
            temporary_context_type: sm.temporary_context_type,
            custom_error: sm.custom_error,
            states,
            starting_state,
            state_data,
            events,
            event_data,
            states_events_mapping,
            eventless_transitions,
            entry_exit_async: sm.entry_exit_async,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_machine(source: &str) -> parse::Result<ParsedStateMachine> {
        ParsedStateMachine::new(syn::parse_str::<StateMachine>(source)?)
    }

    fn machine(source: &str) -> ParsedStateMachine {
        parse_machine(source).unwrap_or_else(|e| panic!("`{}` should parse: {}", source, e))
    }

    fn error(source: &str) -> String {
        match parse_machine(source) {
            Ok(_) => panic!("`{}` should be rejected", source),
            Err(e) => e.to_string(),
        }
    }

    fn sorted_keys<V>(map: &HashMap<String, V>) -> Vec<&str> {
        let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn machine_without_eventless_transitions_has_an_empty_map() {
        let sm = machine("transitions: { *A + E = B, B + F = A }");
        assert!(sm.eventless_transitions.is_empty());
        assert_eq!(sorted_keys(&sm.events), ["E", "F"]);
    }

    #[test]
    fn eventless_transitions_are_collected_in_declaration_order() {
        let sm = machine(
            "transitions: {
                *Idle + Start / configure = Check,
                Check [ok] = Ready,
                Check [async other] / fix = Retry,
                Check = Error,
            }",
        );

        assert_eq!(sorted_keys(&sm.eventless_transitions), ["Check"]);
        let check = &sm.eventless_transitions["Check"];
        assert_eq!(check.len(), 3);
        assert_eq!(check[0].guard.as_ref().unwrap().to_string(), "ok()");
        assert!(check[0].action.is_none());
        assert_eq!(check[0].out_state, "Ready");
        assert_eq!(
            check[1].guard.as_ref().unwrap().to_string(),
            "other().await"
        );
        assert_eq!(check[1].action.as_ref().unwrap().ident, "fix");
        assert_eq!(check[1].out_state, "Retry");
        assert!(check[2].guard.is_none());
        assert_eq!(check[2].out_state, "Error");

        // Eventless transitions do not create events
        assert_eq!(sorted_keys(&sm.events), ["Start"]);
        assert!(sm.event_data.data_types.is_empty());
    }

    #[test]
    fn states_only_in_eventless_transitions_are_still_collected() {
        let sm = machine(
            "transitions: {
                *Idle + Start / configure = Check,
                Check [ok] = Ready,
                Check = Error,
            }",
        );

        // Every state has an entry, also the ones that only appear in eventless transitions
        let expected = ["Check", "Error", "Idle", "Ready"];
        assert_eq!(sorted_keys(&sm.states), expected);
        assert_eq!(sorted_keys(&sm.states_events_mapping), expected);

        // `states_events_mapping` only has event transitions
        assert_eq!(sorted_keys(&sm.states_events_mapping["Idle"]), ["Start"]);
        assert!(sm.states_events_mapping["Check"].is_empty());
        assert!(sm.states_events_mapping["Ready"].is_empty());
        assert!(sm.states_events_mapping["Error"].is_empty());
    }

    #[test]
    fn eventless_state_data_is_collected() {
        let sm = machine(
            "transitions: {
                *Idle + Start / begin = Measuring(u32),
                Measuring(u32) [async ready] / async finish = Done(u8),
                Done(u8) / ack = Idle,
            }",
        );
        assert_eq!(
            sorted_keys(&sm.state_data.data_types),
            ["Done", "Measuring"]
        );
        assert_eq!(
            sorted_keys(&sm.eventless_transitions),
            ["Done", "Measuring"]
        );
        assert_eq!(sorted_keys(&sm.events), ["Start"]);

        // A state that only appears in eventless transitions can have data
        let sm = machine(
            "transitions: {
                *Idle + Start = Check,
                Check [ok] / produce = Hold(u8),
                Hold(u8) / consume = Idle,
            }",
        );
        assert_eq!(sorted_keys(&sm.state_data.data_types), ["Hold"]);
    }

    #[test]
    fn eventless_self_loop_is_collected() {
        let sm = machine("transitions: { *A + E = B, B [done] = A, B [!done] / work = B }");
        let b = &sm.eventless_transitions["B"];
        assert_eq!(b.len(), 2);
        assert_eq!(b[1].out_state, "B");
    }

    #[test]
    fn event_and_eventless_transitions_can_be_mixed() {
        let sm = machine(
            "transitions: {
                *A + E = B,
                B + F = A,
                B [g] = C,
                B = D,
            }",
        );
        assert_eq!(sorted_keys(&sm.states_events_mapping["B"]), ["F"]);
        assert_eq!(sm.eventless_transitions["B"].len(), 2);
        assert_eq!(sorted_keys(&sm.events), ["E", "F"]);
    }

    #[test]
    fn wildcard_expansion_ignores_eventless_transitions() {
        let sm = machine(
            "transitions: {
                *A + E = B,
                B / a = C,
                _ + Reset = A,
            }",
        );

        // The wildcard is expanded for all states, including `C` that is only the target of an
        // eventless transition.
        for state in ["A", "B", "C"] {
            assert_eq!(
                sm.states_events_mapping[state]["Reset"].transitions.len(),
                1,
                "{}",
                state
            );
        }
        assert_eq!(sorted_keys(&sm.states_events_mapping["B"]), ["Reset"]);
        assert_eq!(sorted_keys(&sm.states), ["A", "B", "C"]);

        // The eventless transition is neither expanded nor duplicated
        assert_eq!(sorted_keys(&sm.eventless_transitions), ["B"]);
        assert_eq!(sm.eventless_transitions["B"].len(), 1);
        assert_eq!(sorted_keys(&sm.events), ["E", "Reset"]);
    }

    #[test]
    fn wildcard_without_effect_is_still_reported() {
        let message = error(
            "transitions: {
                *A + E = B,
                _ + E = C,
                B / a = C,
                _ + E = A,
            }",
        );
        assert_eq!(message, "Wildcard has no effect");
    }

    #[test]
    fn starting_state_cannot_have_eventless_transitions() {
        for source in [
            // on the `*` line
            "transitions: { *S1 / a = S2 }",
            "transitions: { *S1 [g] = S2, S2 + E = S1 }",
            // by state name
            "transitions: { *S1 + E = S2, S1 [g] = S3 }",
            "transitions: { *S1 + E = S2, S2 + F = S1, S1 / a = S3 }",
        ] {
            assert_eq!(
                error(source),
                "The starting state cannot have eventless transitions, because `new()` cannot run guards or actions",
                "`{}`",
                source
            );
        }

        // Other states are fine
        machine("transitions: { *S1 + E = S2, S2 [g] = S1 }");
    }

    #[test]
    fn eventless_transition_to_state_with_data_needs_an_action() {
        for source in [
            "transitions: { *S1 + E = S2, S2 [g] = S3(u32) }",
            "transitions: { *S1 + E / a = S3(u32), S2 = S3(u32), S1 + F = S2 }",
        ] {
            assert_eq!(
                error(source),
                "This state has data associated, but not action is define here to provide it.",
                "`{}`",
                source
            );
        }
        machine("transitions: { *S1 + E = S2, S2 [g] / a = S3(u32) }");

        // The same rule for event transitions is unchanged
        assert_eq!(
            error("transitions: { *S1 + E = S2(u32) }"),
            "This state has data associated, but not action is define here to provide it."
        );
    }

    #[test]
    fn eventless_transitions_need_an_explicit_target() {
        // Rejected by the parser, also when parsing a whole machine
        assert_eq!(
            error("transitions: { *A + E = B, B, }"),
            "expected `+ Event`, or an explicit target state (`= State`) for an eventless transition"
        );
        assert_eq!(
            error("transitions: { *A + E = B, _ = B }"),
            "Eventless transitions cannot use a wildcard input state; add an event (`_ + Event`)"
        );
    }
}
