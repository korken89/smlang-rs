use crate::parser::*;

/// Generates a string containing 'dot' syntax to generate a statemachine diagram with graphviz.
pub fn generate_diagram(sm: &ParsedStateMachine) -> String {
    let transitions = &sm.states_events_mapping;

    let mut diagram_states = sm.states.iter().map(|s| s.0).collect::<Vec<&String>>();
    diagram_states.sort();
    let diagram_states = diagram_states.into_iter();
    let mut diagram_events = vec![];
    let mut diagram_transitions = vec![];
    for (state, event) in transitions {
        for (_event, eventmapping) in event {
            diagram_events.push((
                eventmapping.event.to_string(),
                eventmapping
                    .guard
                    .as_ref()
                    .map(|i| i.ident.to_string())
                    .unwrap_or_else(|| "_".to_string()),
                eventmapping
                    .action
                    .as_ref()
                    .map(|i| i.ident.to_string())
                    .unwrap_or_else(|| "_".to_string()),
            ));
            diagram_transitions.push((
                state,
                eventmapping.out_state.to_string(),
                eventmapping.event.to_string(),
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
    let transition_string = diagram_transitions
        .iter()
        .map(|t| format!("\t{0} -> {1} [color=blue label={2}];", t.0, t.1, t.2))
        .collect::<Vec<String>>();

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
        sm.starting_state.to_string(),
        state_string.join("\n"),
        event_string.join("\n"),
        transition_string.join("\n")
    )
}
