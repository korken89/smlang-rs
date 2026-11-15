use crate::parser::event::Transition;
use crate::parser::*;

/// The guard of a transition as shown in the diagram, `_` if there is none.
fn guard_label(transition: &Transition) -> String {
    transition
        .guard
        .as_ref()
        .map(|i| i.to_string())
        .unwrap_or_else(|| "_".to_string())
}

/// The action of a transition as shown in the diagram, `_` if there is none.
fn action_label(transition: &Transition) -> String {
    transition
        .action
        .as_ref()
        .map(|i| i.ident.to_string())
        .unwrap_or_else(|| "_".to_string())
}

/// Generates a string containing 'dot' syntax to generate a statemachine diagram with graphviz.
pub fn generate_diagram(sm: &ParsedStateMachine) -> String {
    let transitions = &sm.states_events_mapping;

    let mut diagram_states = sm.states.iter().map(|s| s.0).collect::<Vec<&String>>();
    diagram_states.sort();
    let diagram_states = diagram_states.into_iter();
    let mut diagram_events = vec![];
    let mut diagram_transitions = vec![];
    for (state, event) in transitions {
        for eventmapping in event.values() {
            for transition in &eventmapping.transitions {
                diagram_events.push((
                    eventmapping.event.to_string(),
                    guard_label(transition),
                    action_label(transition),
                ));
                diagram_transitions.push((
                    state,
                    transition.out_state.to_string(),
                    eventmapping.event.to_string(),
                ));
            }
        }
    }

    // Eventless transitions have no event, so there is no event node for them. They are drawn
    // as dashed edges, labelled with their guard and action. The first transition of a state
    // whose guard passes is taken, so if a state has several, the label starts with the 1-based
    // position in declaration order. The layout of the graph does not show the order of edges.
    // They are kept in declaration order, which is stable, so they need no sorting.
    let mut diagram_eventless_transitions = vec![];
    for (state, transitions) in &sm.eventless_transitions {
        for (index, transition) in transitions.iter().enumerate() {
            let priority = if transitions.len() > 1 {
                format!("{}: ", index + 1)
            } else {
                String::new()
            };
            diagram_eventless_transitions.push((
                state,
                transition.out_state.to_string(),
                guard_label(transition),
                action_label(transition),
                priority,
            ));
        }
    }

    // Sorting is needed to ensure stable (ie not changing between runs of
    // the same sm code) dot file contents. This is needed to ensure stable
    // hash sum, which is used to name unnamed diagrams. If done without sorting,
    // the output is polluted with lots of similar svg files with different names.
    // This ensures that new files will only occur upon changing the structure of the code.
    diagram_events.sort();
    diagram_transitions.sort();

    let state_string = diagram_states
        .map(|s| {
            format!(
                "\t{} [shape=box color=\"red\" fillcolor=\"#ffbb33\" style=filled]",
                s
            )
        })
        .collect::<Vec<String>>();
    let event_string = diagram_events
        .iter()
        .map(|s| {
            format!(
                "\t{0} [shape=box label=\"{0}\\n[{1}] / {2}\"]",
                s.0, s.1, s.2
            )
        })
        .collect::<Vec<String>>();
    let mut transition_string = diagram_transitions
        .iter()
        .map(|t| format!("\t{0} -> {1} [color=blue label={2}];", t.0, t.1, t.2))
        .collect::<Vec<String>>();
    transition_string.extend(diagram_eventless_transitions.iter().map(|t| {
        format!(
            "\t{0} -> {1} [color=blue style=dashed label=\"{4}[{2}] / {3}\"];",
            t.0, t.1, t.2, t.3, t.4
        )
    }));

    format!(
        "digraph G {{
    rankdir=\"LR\";
    node [fontname=Arial];
    edge [fontname=Arial];
    s [shape=circle size=2 color=\"black\" style=filled]
    
    s -> {}
{}

{}

{}
}}",
        sm.starting_state,
        state_string.join("\n"),
        event_string.join("\n"),
        transition_string.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagram(source: &str) -> String {
        generate_diagram(&parse_machine(source).unwrap())
    }

    #[test]
    fn eventless_transitions_are_dashed_edges() {
        let dot = diagram(
            "transitions: {
                *Idle + Start / configure = Check,
                Check [ok && !failed] = Ready,
                Check / recover = Retry,
                Check = Error,
                Retry = Done,
            }",
        );

        // Eventless edges are dashed and labelled `[guard] / action`, `_` is a missing guard or
        // action. They are in declaration order, grouped by source state. `Check` has several,
        // so the labels start with the position, which decides what fires first. `Retry` has a
        // single one, which gets no position.
        let eventless: Vec<&str> = dot.lines().filter(|l| l.contains("style=dashed")).collect();
        assert_eq!(
            eventless,
            [
                "\tCheck -> Ready [color=blue style=dashed label=\"1: [ok() && !failed()] / _\"];",
                "\tCheck -> Retry [color=blue style=dashed label=\"2: [_] / recover\"];",
                "\tCheck -> Error [color=blue style=dashed label=\"3: [_] / _\"];",
                "\tRetry -> Done [color=blue style=dashed label=\"[_] / _\"];",
            ]
        );

        // The event transition is drawn as before, and eventless transitions get no event node
        assert!(dot.contains("\tIdle -> Check [color=blue label=Start];"));
        assert!(dot.contains("\tStart [shape=box label=\"Start\\n[_] / configure\"]"));
        assert_eq!(dot.matches("[shape=box label=").count(), 1);
    }

    #[test]
    fn diagram_is_deterministic() {
        let source = "transitions: {
                *A + E = B,
                B [g] = C,
                B / a = D,
                C [h] = A,
                C = D,
                D + F = A,
            }";
        let first = diagram(source);
        for _ in 0..10 {
            assert_eq!(diagram(source), first);
        }
    }
}
