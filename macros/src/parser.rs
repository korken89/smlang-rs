use proc_macro2::Span;
use std::collections::HashMap;
use syn::{bracketed, parenthesized, parse, spanned::Spanned, token, Ident, Token, Type};

#[derive(Debug)]
pub struct StateMachine {
    pub transitions: Vec<StateTransition>,
}

impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            transitions: Vec::new(),
        }
    }

    pub fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
    }
}

#[derive(Debug)]
pub struct EventMapping {
    pub event: Ident,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
    pub out_state: Ident,
}

#[derive(Debug)]
pub struct ParsedStateMachine {
    pub states: HashMap<String, Ident>,
    pub starting_state: Ident,
    pub state_data_type: HashMap<String, Type>,
    pub events: HashMap<String, Ident>,
    pub event_data_type: HashMap<String, Type>,
    pub states_events_mapping: HashMap<String, HashMap<String, EventMapping>>,
}

impl ParsedStateMachine {
    pub fn new(sm: StateMachine) -> parse::Result<Self> {
        // Check the initial state definition
        let num_start: usize = sm
            .transitions
            .iter()
            .map(|sm| if sm.start { 1 } else { 0 })
            .sum();

        if num_start == 0 {
            return Err(parse::Error::new(
                Span::call_site(),
                "No starting state defined, indicate the starting state with a *",
            ));
        } else if num_start > 1 {
            return Err(parse::Error::new(
                Span::call_site(),
                "More than one starting state defined (indicated with *), remove duplicates",
            ));
        }

        // Extract the starting state
        let starting_state = sm
            .transitions
            .iter()
            .find(|sm| sm.start)
            .unwrap()
            .in_state
            .clone();

        let mut states = HashMap::new();
        let mut state_data_type = HashMap::new();
        let mut events = HashMap::new();
        let mut event_data_type = HashMap::new();
        let mut states_events_mapping = HashMap::<String, HashMap<String, EventMapping>>::new();

        for transition in sm.transitions.iter() {
            // Collect states
            states.insert(transition.in_state.to_string(), transition.in_state.clone());
            states.insert(
                transition.out_state.to_string(),
                transition.out_state.clone(),
            );

            // Collect state to data mappings and check for definition errors
            if let Some(state_type) = transition.state_data_type.clone() {
                match state_data_type.get(&transition.in_state.to_string()) {
                    None => {
                        state_data_type.insert(transition.in_state.to_string(), state_type);
                    }
                    Some(v) => {
                        if v != &state_type {
                            return Err(parse::Error::new(
                                transition.in_state.span(),
                                "This state's type does not match its previous definition.",
                            ));
                        }
                    }
                }
            } else if let Some(_) = state_data_type.get(&transition.event.to_string()) {
                return Err(parse::Error::new(
                    transition.event.span(),
                    "This event's type does not match its previous definition.",
                ));
            }

            // Collect events
            events.insert(transition.event.to_string(), transition.event.clone());

            // Collect event to data mappings and check for definition errors
            if let Some(event_type) = transition.event_data_type.clone() {
                match event_data_type.get(&transition.event.to_string()) {
                    None => {
                        event_data_type.insert(transition.event.to_string(), event_type);
                    }
                    Some(v) => {
                        if v != &event_type {
                            return Err(parse::Error::new(
                                transition.event.span(),
                                "This event's type does not match its previous definition.",
                            ));
                        }
                    }
                }
            } else if let Some(_) = event_data_type.get(&transition.event.to_string()) {
                return Err(parse::Error::new(
                    transition.event.span(),
                    "This event's type does not match its previous definition.",
                ));
            }

            // Setup the states to events mapping
            states_events_mapping.insert(transition.in_state.to_string(), HashMap::new());
        }

        // Create states to event mappings
        for transition in sm.transitions.iter() {
            let p = states_events_mapping
                .get_mut(&transition.in_state.to_string())
                .unwrap();

            if let None = p.get(&transition.event.to_string()) {
                let mapping = EventMapping {
                    event: transition.event.clone(),
                    guard: transition.guard.clone(),
                    action: transition.action.clone(),
                    out_state: transition.out_state.clone(),
                };

                p.insert(transition.event.to_string(), mapping);
            } else {
                return Err(parse::Error::new(
                    transition.in_state.span(),
                    "State and event combination specified multiple times, remove duplicates",
                ));
            }
        }

        Ok(ParsedStateMachine {
            states,
            starting_state,
            state_data_type,
            events,
            event_data_type,
            states_events_mapping,
        })
    }
}

#[derive(Debug)]
pub struct StateTransition {
    start: bool,
    in_state: Ident,
    state_data_type: Option<Type>,
    event: Ident,
    event_data_type: Option<Type>,
    guard: Option<Ident>,
    action: Option<Ident>,
    out_state: Ident,
}

impl parse::Parse for StateMachine {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let mut statemachine = StateMachine::new();

        loop {
            // If the last line ends with a comma this is true
            if input.is_empty() {
                break;
            }

            // Check for starting state definition
            let start = if let Ok(_) = input.parse::<Token![*]>() {
                true
            } else {
                false
            };

            //
            // Parse the DSL
            //
            // Transition DSL:
            // SrcState(OptionalType1) + Event(OptionalType2) [ guard ] / action = DstState

            // Input State
            let in_state: Ident = input.parse()?;

            // Possible type on the state
            let state_data_type = if input.peek(token::Paren) {
                let content;
                parenthesized!(content in input);
                let input: Type = content.parse()?;

                // Check if this is the starting state, it cannot have data as there is no
                // supported way of propagating it (for now)
                if start {
                    return Err(parse::Error::new(
                        input.span(),
                        "The starting state cannot have data associated with it.",
                    ));
                }

                // Check so the type is supported
                match &input {
                    Type::Array(_)
                    | Type::Path(_)
                    | Type::Ptr(_)
                    | Type::Reference(_)
                    | Type::Slice(_)
                    | Type::Tuple(_) => (),
                    _ => {
                        return Err(parse::Error::new(
                            input.span(),
                            "This is an unsupported type for states.",
                        ))
                    }
                }

                Some(input)
            } else {
                None
            };

            // Event
            input.parse::<Token![+]>()?;
            let event: Ident = input.parse()?;

            // Possible type on the event
            let event_data_type = if input.peek(token::Paren) {
                let content;
                parenthesized!(content in input);
                let input: Type = content.parse()?;

                // Check so the type is supported
                match &input {
                    Type::Array(_)
                    | Type::Path(_)
                    | Type::Ptr(_)
                    | Type::Reference(_)
                    | Type::Slice(_)
                    | Type::Tuple(_) => (),
                    _ => {
                        return Err(parse::Error::new(
                            input.span(),
                            "This is an unsupported type for events.",
                        ))
                    }
                }

                Some(input)
            } else {
                None
            };

            // Possible guard
            let guard = if input.peek(token::Bracket) {
                let content;
                bracketed!(content in input);
                let guard: Ident = content.parse()?;
                Some(guard)
            } else {
                None
            };

            // Possible action
            let action = if let Ok(_) = input.parse::<Token![/]>() {
                let action: Ident = input.parse()?;
                Some(action)
            } else {
                None
            };

            input.parse::<Token![=]>()?;

            let out_state: Ident = input.parse()?;

            statemachine.add_transition(StateTransition {
                start,
                in_state,
                state_data_type,
                event,
                event_data_type,
                guard,
                action,
                out_state,
            });

            // No comma at end of line, no more transitions
            if input.is_empty() {
                break;
            }

            if let Err(_) = input.parse::<Token![,]>() {
                break;
            };
        }

        Ok(statemachine)
    }
}
