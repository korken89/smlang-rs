use crate::parser::event::Transition;
use crate::parser::transition::visit_guards;
use crate::parser::{AsyncIdent, ParsedStateMachine};
use proc_macro2::Span;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use syn::parse;

/// A basic representation an action call signature.
#[derive(PartialEq, Clone)]
struct FunctionSignature {
    // Function input arguments.
    arguments: Vec<syn::Type>,

    // Function result (if any).
    result: Option<syn::Type>,

    // Is the function async
    is_async: bool,
}

impl FunctionSignature {
    pub fn new(
        input_data: Option<&syn::Type>,
        event_data: Option<&syn::Type>,
        output_data: Option<&syn::Type>,
        is_async: bool,
    ) -> Self {
        let mut input_arguments = vec![];

        if let Some(datatype) = input_data {
            input_arguments.push(datatype.clone());
        }

        if let Some(datatype) = event_data {
            input_arguments.push(datatype.clone());
        }

        let result = output_data.cloned();

        Self {
            arguments: input_arguments,
            result,
            is_async,
        }
    }

    pub fn new_guard(
        input_state: Option<&syn::Type>,
        event: Option<&syn::Type>,
        is_async: bool,
    ) -> Self {
        // Guards never have output data.
        Self::new(input_state, event, None, is_async)
    }
}

/// The data of the transition context, as seen by the guards and actions of a transition.
struct TransitionData<'a> {
    /// The data of the source state.
    in_state: Option<&'a syn::Type>,
    /// The data of the event. Always `None` for eventless transitions.
    event: Option<&'a syn::Type>,
}

/// Visit all transitions of the state machine, event transitions and eventless transitions.
///
/// Event transitions are visited with the data of their event. Eventless transitions are
/// visited without event data. The data of the output state is looked up per transition from
/// `Transition::out_state`.
fn visit_transitions<F>(sm: &ParsedStateMachine, mut visit: F) -> Result<(), parse::Error>
where
    F: FnMut(&TransitionData, &Transition) -> Result<(), parse::Error>,
{
    for (in_state, event, transition) in sm.all_transitions() {
        let data = TransitionData {
            in_state: sm.state_data.data_types.get(in_state),
            event: event.and_then(|event| sm.event_data.data_types.get(event)),
        };
        visit(&data, transition)?;
    }

    Ok(())
}

// Verify action and guard function signatures.
fn validate_action_signatures(sm: &ParsedStateMachine) -> Result<(), parse::Error> {
    // Collect all of the action call signatures.
    let mut actions = HashMap::new();

    visit_transitions(sm, |data, transition| {
        if let Some(AsyncIdent {
            ident: action,
            is_async,
        }) = &transition.action
        {
            // Internal transitions are already resolved to their input state, so the
            // output state is always a concrete state here.
            let out_state_data = sm
                .state_data
                .data_types
                .get(&transition.out_state.to_string());

            let signature =
                FunctionSignature::new(data.in_state, data.event, out_state_data, *is_async);

            // If the action is not yet known, add it to our tracking list.
            actions
                .entry(action.to_string())
                .or_insert_with(|| signature.clone());

            // Check that the call signature is equivalent to the recorded signature for this
            // action.
            if actions.get(&action.to_string()).unwrap() != &signature {
                return Err(parse::Error::new(
                    Span::call_site(),
                    format!("Action `{}` can only be reused when all input states, events, and output states have the same data", action),
                ));
            }
        }
        Ok(())
    })
}

fn validate_guard_signatures(sm: &ParsedStateMachine) -> Result<(), parse::Error> {
    // Collect all of the guard call signatures.
    let mut guards = HashMap::new();

    visit_transitions(sm, |data, transition| {
        if let Some(guard_expression) = &transition.guard {
            visit_guards(guard_expression, |guard| {
                let signature =
                    FunctionSignature::new_guard(data.in_state, data.event, guard.is_async);

                // If the action is not yet known, add it to our tracking list.
                guards
                    .entry(guard.ident.to_string())
                    .or_insert_with(|| signature.clone());

                // Check that the call signature is equivalent to the recorded signature for this
                // guard.
                if guards.get(&guard.ident.to_string()).unwrap() != &signature {
                    return Err(parse::Error::new(
                        Span::call_site(),
                        format!("Guard `{}` can only be reused when all input states and events have the same data", guard.ident),
                    ));
                }
                Ok(())
            })?;
        }
        Ok(())
    })
}

/// Check the transitions of one source state (and one event, if any) for transitions that can
/// never be taken.
///
/// There can be at most one unguarded transition, and no guarded transition can follow it.
/// The error messages name the source state and the event, or say that the transitions are
/// eventless. Errors of eventless transitions point to the offending transition, errors of event
/// transitions keep the span of the macro invocation.
fn check_reachability(
    in_state: &str,
    event: Option<&str>,
    transitions: &[Transition],
) -> Result<(), parse::Error> {
    let label = || match event {
        Some(event) => format!("{} + {}", in_state, event),
        None => format!("{} (eventless)", in_state),
    };
    let duplicate = match event {
        Some(_) => "State and event combination",
        None => "Unguarded eventless transition",
    };
    let span_of = |t: &Transition| match event {
        Some(_) => Span::call_site(),
        None => t.out_state.span(),
    };

    let mut unguarded_count = 0;
    for t in transitions {
        if let Some(g) = &t.guard {
            if unguarded_count > 0 {
                // Guarded transition AFTER an unguarded one
                return Err(parse::Error::new(
                    span_of(t),
                    format!("{}: [{}] : guarded transition is unreachable because it follows an unguarded transition, which handles all cases",
                            label(), g),
                ));
            }
        } else {
            // unguarded
            unguarded_count += 1;
            if unguarded_count > 1 {
                return Err(parse::Error::new(
                    span_of(t),
                    format!(
                        "{}: {} specified multiple times, remove duplicates.",
                        label(),
                        duplicate
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn validate_unreachable_transitions(sm: &ParsedStateMachine) -> Result<(), parse::Error> {
    let all_transitions = &sm.states_events_mapping;
    for (in_state, event_mappings) in all_transitions {
        for (event, event_mapping) in event_mappings {
            // more than single transition for (in_state,event)
            if event_mapping.transitions.len() > 1 {
                check_reachability(in_state, Some(event), &event_mapping.transitions)?;
            }
        }
    }

    // The eventless transitions of a state are evaluated in declaration order, like the
    // transitions of an event. The states are sorted, so errors are reported deterministically.
    for (in_state, transitions) in &sm.eventless_transitions {
        check_reachability(in_state, None, transitions)?;
    }
    Ok(())
}

/// Reject eventless transitions that cannot stop.
///
/// An unguarded eventless transition always fires, so a state that has one can never rest.
/// Eventless processing can only stop in a state without an unguarded eventless transition.
/// A state with an unguarded eventless transition is *doomed* if every eventless transition
/// leads to a doomed state, guarded or not: once it is entered, the machine loops forever
/// whatever the guards return. This is the self-loop `S / a = S`, the cycle
/// `A = B, B = A`, and also `A [g] = B, A = B, B = A`, where the guard only leads back into
/// the loop.
///
/// A loop that has a guarded way out is accepted, because guarded transitions are evaluated
/// before the unguarded one: `S [done] = T, S / work = S` is a valid `while !done { work }`.
fn validate_eventless_cycles(sm: &ParsedStateMachine) -> Result<(), parse::Error> {
    // The names of the targets of the eventless transitions of each state.
    let targets: BTreeMap<&str, Vec<String>> = sm
        .eventless_transitions
        .iter()
        .map(|(in_state, transitions)| {
            let names = transitions.iter().map(|t| t.out_state.to_string());
            (in_state.as_str(), names.collect())
        })
        .collect();

    // After `validate_unreachable_transitions` every state has at most one unguarded eventless
    // transition. Take the first one to stay independent of the validation order. `edges` holds
    // the index of that transition.
    let edges: BTreeMap<&str, usize> = sm
        .eventless_transitions
        .iter()
        .filter_map(|(in_state, transitions)| {
            let index = transitions.iter().position(|t| t.guard.is_none())?;
            Some((in_state.as_str(), index))
        })
        .collect();

    // Start with every state that has an unguarded eventless transition, and repeatedly remove
    // the states that have an eventless transition to a state that is not doomed. What is left
    // at the fixpoint can never stop.
    let mut doomed: BTreeSet<&str> = edges.keys().copied().collect();
    loop {
        let rescued: Vec<&str> = doomed
            .iter()
            .copied()
            .filter(|state| targets[state].iter().any(|t| !doomed.contains(t.as_str())))
            .collect();
        if rescued.is_empty() {
            break;
        }
        for state in &rescued {
            doomed.remove(state);
        }
    }

    // Report the cycle that is reachable from the smallest doomed state, so the message is
    // stable across builds. The unguarded edge of a doomed state stays inside the doomed states
    // (otherwise it would have been rescued), so the walk ends in a cycle.
    if let Some(&start) = doomed.iter().next() {
        let mut path = vec![start];
        let cycle = loop {
            let last = path[path.len() - 1];
            let next = targets[last][edges[last]].as_str();
            match path.iter().position(|&state| state == next) {
                Some(position) => break path.split_off(position),
                None => path.push(next),
            }
        };

        let names: Vec<&str> = cycle.iter().copied().chain([cycle[0]]).collect();
        let first_edge = &sm.eventless_transitions[cycle[0]][edges[cycle[0]]];
        return Err(parse::Error::new(
            first_edge.out_state.span(),
            format!(
                "Unguarded eventless transitions form an infinite loop: {} (no guarded eventless transition leads out of it)",
                names.join(" -> ")
            ),
        ));
    }
    Ok(())
}

/// Validate coherency of the state machine.
pub fn validate(sm: &ParsedStateMachine) -> Result<(), parse::Error> {
    validate_action_signatures(sm)?;
    validate_guard_signatures(sm)?;
    validate_unreachable_transitions(sm)?;
    validate_eventless_cycles(sm)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;
    use crate::parser::parse_machine;
    use syn::parse;

    fn check(transitions: &str) -> parse::Result<()> {
        let source = format!("transitions: {{ {} }}", transitions);
        validate(&parse_machine(&source)?)
    }

    fn accepted(transitions: &str) {
        if let Err(e) = check(transitions) {
            panic!("`{}` should be valid: {}", transitions, e);
        }
    }

    fn rejected(transitions: &str) -> String {
        match check(transitions) {
            Ok(()) => panic!("`{}` should be rejected", transitions),
            Err(e) => e.to_string(),
        }
    }

    const ACTION_REUSE: &str = "Action `a` can only be reused when all input states, events, and output states have the same data";
    const GUARD_REUSE: &str =
        "Guard `g` can only be reused when all input states and events have the same data";

    #[test]
    fn action_reuse_between_event_and_eventless_transition() {
        // Same signature: no input data, no event data, no output data
        accepted("*S1 + E / a = S2, S2 / a = S3");
        // Same signature with state data on both sides
        accepted("*S1 + E / mk = S2(u8), S2(u8) + F / a = S3, S2(u8) / a = S4");

        // An event with data is an extra argument that eventless transitions do not have
        assert_eq!(rejected("*S1 + E(u8) / a = S2, S2 / a = S3"), ACTION_REUSE);
        assert_eq!(
            rejected("*S1 + E / a = S2, S2 / a = S3, S3 + F(u8) / a = S1"),
            ACTION_REUSE
        );
        // Input state data
        assert_eq!(
            rejected("*S1 + E / mk = S2(u8), S2(u8) / a = S3, S3 + F / a = S4"),
            ACTION_REUSE
        );
        // Async and sync actions have different signatures
        assert_eq!(
            rejected("*S1 + E / a = S2, S2 / async a = S3"),
            ACTION_REUSE
        );
    }

    #[test]
    fn action_output_data_is_looked_up_per_eventless_transition() {
        // The output data of `a` differs between the two eventless transitions
        assert_eq!(
            rejected("*S1 + E = S2, S2 [g] / a = S3(u8), S2 / a = S4"),
            ACTION_REUSE
        );
        // ... and between an event transition and an eventless transition
        assert_eq!(
            rejected("*S1 + E / a = S2(u8), S2(u8) / a = S3(u16)"),
            ACTION_REUSE
        );
        accepted("*S1 + E = S2, S2 [g] / a = S3(u8), S2 [h] / a = S4(u8), S2 = S1");
    }

    #[test]
    fn guard_reuse_between_event_and_eventless_transition() {
        accepted("*S1 + E [g] = S2, S2 [g] = S3, S2 = S4");
        accepted("*S1 + E / mk = S2(u8), S2(u8) + F [g] = S3, S2(u8) [g] = S4, S2(u8) = S1");
        assert_eq!(rejected("*S1 + E(u8) [g] = S2, S2 [g] = S3"), GUARD_REUSE);
        assert_eq!(
            rejected("*S1 + E [g] / mk = S2(u8), S2(u8) [g] = S3"),
            GUARD_REUSE
        );
        assert_eq!(rejected("*S1 + E [g] = S2, S2 [async g] = S3"), GUARD_REUSE);
        // Guards inside of expressions are covered too
        assert_eq!(
            rejected("*S1 + E(u8) [g] = S2, S2 [!h && (i || g)] = S3"),
            GUARD_REUSE
        );
    }

    #[test]
    fn duplicate_unguarded_eventless_transitions() {
        assert_eq!(
            rejected("*S1 + E = S2, S2 = S3, S2 = S4"),
            "S2 (eventless): Unguarded eventless transition specified multiple times, remove duplicates."
        );
        assert_eq!(
            rejected("*S1 + E = S2, S2 [g] = S3, S2 = S4, S2 / a = S5"),
            "S2 (eventless): Unguarded eventless transition specified multiple times, remove duplicates."
        );
    }

    #[test]
    fn guarded_eventless_transition_after_unguarded() {
        assert_eq!(
            rejected("*S1 + E = S2, S2 = S3, S2 [g] = S4"),
            "S2 (eventless): [g()] : guarded transition is unreachable because it follows an unguarded transition, which handles all cases"
        );
        accepted("*S1 + E = S2, S2 [g] = S3, S2 [h] = S4, S2 = S1");
    }

    #[test]
    fn event_and_eventless_transitions_on_the_same_state() {
        // The event transitions and the eventless transitions are checked separately
        accepted("*S1 + E = S2, S2 + F = S1, S2 [g] = S3, S2 = S4");
        accepted("*S1 + E = S2, S2 + F [g] = S3, S2 + F = S1, S2 [h] = S3, S2 = S4");
        // Existing event rules are unchanged
        assert_eq!(
            rejected("*S1 + E = S2, S2 + F = S1, S2 + F = S3, S2 = S4"),
            "S2 + F: State and event combination specified multiple times, remove duplicates."
        );
    }

    fn cycle_error(cycle: &str) -> String {
        format!(
            "Unguarded eventless transitions form an infinite loop: {} (no guarded eventless transition leads out of it)",
            cycle
        )
    }

    #[test]
    fn unguarded_eventless_cycles_are_rejected() {
        assert_eq!(
            rejected("*S + E = A, A / x = B, B = A"),
            cycle_error("A -> B -> A")
        );
        assert_eq!(rejected("*S + E = A, A / x = A"), cycle_error("A -> A"));
        assert_eq!(
            rejected("*S + E = A, A = B, B = C, C = A"),
            cycle_error("A -> B -> C -> A")
        );
        // Guards on the other alternatives of an event transition do not matter
        assert_eq!(
            rejected("*S + E = A, A + F [g] = S, A = A"),
            cycle_error("A -> A")
        );
        // Only the cycle is reported, not the unguarded transitions that lead into it
        assert_eq!(
            rejected("*S + E = X, X = A, A = B, B = A"),
            cycle_error("A -> B -> A")
        );
    }

    #[test]
    fn guarded_transitions_back_into_the_loop_do_not_leave_it() {
        // Whatever `g` returns, the machine ends up in the loop `A -> B -> A`
        assert_eq!(
            rejected("*S + E = A, A [g] = B, A = B, B = A"),
            cycle_error("A -> B -> A")
        );
        // The guarded transition leads to another state that cannot stop either
        assert_eq!(
            rejected("*S + E = A, A [g] = C, C = A, A = B, B = A"),
            cycle_error("A -> B -> A")
        );
        assert_eq!(
            rejected("*S + E = A, A [g] = B, A = B, B [h] = A, B = A"),
            cycle_error("A -> B -> A")
        );
        // A self-loop with a guarded transition into itself
        assert_eq!(
            rejected("*S + E = A, A [g] / x = A, A / y = A"),
            cycle_error("A -> A")
        );
    }

    #[test]
    fn eventless_cycles_with_a_guarded_way_out_are_accepted() {
        // Retry loop: the guard of `A` can leave it before the unguarded cycle is followed
        accepted("*S + E = A, A [g] = X, A / retry = B, B = A");
        // The same thing as a guarded self-loop: while !done { work }
        accepted("*S + E = A, A [done] = T, A / work = A");
        accepted("*S + E = A, A [!done] / work = A, A = T");
        // `A` can rest when `g` fails, so the unguarded `B = A` does not loop forever
        accepted("*S + E = A, A [g] = B, B = A");
        // A guarded-only cycle never has to continue
        accepted("*S + E = A, A [g] = B, B [h] = A");
        // The way out can be on any state of the cycle
        accepted("*S + E = A, A / x = B, B [g] = T, B = A");
        accepted("*S + E = A, A [g] = T, A / x = B, B = C, C = A");
        accepted("*S + E = A, A [g] = B, A = B, B [h] = T, B = A");

        // Guarded cycles are the responsibility of the user
        accepted("*S + E = A, A [g] = B, A = T, B [h] = A, B = T");
        accepted("*S + E = A, A [g] / x = A, A = T");
    }

    #[test]
    fn only_the_cycle_is_reported() {
        // The tail `Z -> B` leads into the cycle `B -> C -> B`
        assert_eq!(
            rejected("*S + E = Z, Z = B, B = C, C = B"),
            cycle_error("B -> C -> B")
        );
        // `A` is the smallest doomed state and leads into the cycle `B -> C -> B`
        assert_eq!(
            rejected("*S + E = A, A = B, B = C, C = B"),
            cycle_error("B -> C -> B")
        );
    }

    #[test]
    fn the_reported_cycle_is_deterministic() {
        // Two cycles, the smallest doomed state is `A`
        for _ in 0..50 {
            assert_eq!(
                rejected("*S + E = Y, Y = Z, Z = Y, A = B, B = C, C = A, S + F = A"),
                cycle_error("A -> B -> C -> A")
            );
            assert_eq!(
                rejected("*S + E = Y, Y = Z, Z = Y, D = D, S + F = D"),
                cycle_error("D -> D")
            );
        }
    }

    #[test]
    fn acyclic_unguarded_eventless_transitions_are_accepted() {
        accepted("*S + E = A, A = B, B = C");
        accepted("*S + E = A, A / x = B, B / y = C, C [g] = A, C = D");
        // Diamonds are not cycles
        accepted("*S + E = A, A [g] = B, A = C, B = D, C = D");
    }

    #[test]
    fn an_escapable_cycle_does_not_hide_an_infinite_loop() {
        // `A` has a way out, but the loop on `B` has none
        assert_eq!(
            rejected("*S + E = A, A [g] = T, A / w = A, B = B, S + F = B"),
            cycle_error("B -> B")
        );
        assert_eq!(
            rejected("*S + E = Z, Z [g] = T, Z / w = Z, B = B, S + F = B"),
            cycle_error("B -> B")
        );
    }
}
