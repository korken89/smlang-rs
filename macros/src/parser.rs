use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    braced, bracketed, parenthesized, parse, spanned::Spanned, token, GenericArgument, Ident,
    Lifetime, PathArguments, Token, Type,
};

#[derive(Debug)]
pub struct StateMachine {
    pub temporary_context_type: Option<Type>,
    pub guard_error: Option<Type>,
    pub transitions: Vec<StateTransition>,
}

impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            temporary_context_type: None,
            guard_error: None,
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
    pub temporary_context_type: Option<Type>,
    pub guard_error: Option<Type>,
    pub states: HashMap<String, Ident>,
    pub starting_state: Ident,
    pub state_data_type: HashMap<String, Type>,
    pub events: HashMap<String, Ident>,
    pub event_data_type: HashMap<String, Type>,
    pub all_event_data_lifetimes: Vec<Lifetime>,
    pub event_data_lifetimes: HashMap<String, Vec<Lifetime>>,
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
                "No starting state defined, indicate the starting state with a *.",
            ));
        } else if num_start > 1 {
            return Err(parse::Error::new(
                Span::call_site(),
                "More than one starting state defined (indicated with *), remove duplicates.",
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
        let mut all_event_data_lifetimes = Vec::new();
        let mut event_data_lifetimes = HashMap::new();
        let mut states_events_mapping = HashMap::<String, HashMap<String, EventMapping>>::new();

        for transition in sm.transitions.iter() {
            // Collect states
            states.insert(transition.in_state.to_string(), transition.in_state.clone());
            states.insert(
                transition.out_state.to_string(),
                transition.out_state.clone(),
            );

            // Collect state to data mappings and check for definition errors
            if let Some(state_type) = transition.in_state_data_type.clone() {
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

            if let Some(state_type) = transition.out_state_data_type.clone() {
                match state_data_type.get(&transition.out_state.to_string()) {
                    None => {
                        state_data_type.insert(transition.out_state.to_string(), state_type);
                    }
                    Some(v) => {
                        if v != &state_type {
                            return Err(parse::Error::new(
                                transition.out_state.span(),
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
                        let mut lifetimes = Vec::new();
                        match &event_type {
                            Type::Reference(tr) => {
                                if let Some(lifetime) = &tr.lifetime {
                                    lifetimes.push(lifetime.clone());
                                } else {
                                    return Err(parse::Error::new(
                                    transition.event_data_type.span(),
                                    "This event's data lifetime is not defined, consider adding a lifetime.",
                                ));
                                }
                            }
                            Type::Path(tp) => {
                                let punct = &tp.path.segments;
                                for p in punct.iter() {
                                    if let PathArguments::AngleBracketed(abga) = &p.arguments {
                                        for arg in &abga.args {
                                            if let GenericArgument::Lifetime(lifetime) = &arg {
                                                lifetimes.push(lifetime.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                        event_data_type.insert(transition.event.to_string(), event_type);
                        if !lifetimes.is_empty() {
                            event_data_lifetimes
                                .insert(transition.event.to_string(), lifetimes.clone());
                        }
                        all_event_data_lifetimes.append(&mut lifetimes);
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

        // Remove duplicate lifetimes
        all_event_data_lifetimes.dedup();

        for transition in sm.transitions.iter() {
            // Add transitions
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
                    "State and event combination specified multiple times, remove duplicates.",
                ));
            }

            // Check for actions when states have data a
            if let Some(_) = state_data_type.get(&transition.out_state.to_string()) {
                // This transition goes to a state that has data associated, check so it has an
                // action

                if transition.action.is_none() {
                    return Err(parse::Error::new(
                     transition.out_state.span(),
                     "This state has data associated, but not action is define here to provide it.",
                 ));
                }
            }
        }

        // Check so all states with data associated have actions that provide this data

        Ok(ParsedStateMachine {
            temporary_context_type: sm.temporary_context_type,
            guard_error: sm.guard_error,
            states,
            starting_state,
            state_data_type,
            events,
            event_data_type,
            all_event_data_lifetimes,
            event_data_lifetimes,
            states_events_mapping,
        })
    }
}

#[derive(Debug)]
pub struct StateTransition {
    start: bool,
    in_state: Ident,
    in_state_data_type: Option<Type>,
    event: Ident,
    event_data_type: Option<Type>,
    guard: Option<Ident>,
    action: Option<Ident>,
    out_state: Ident,
    out_state_data_type: Option<Type>,
}

impl parse::Parse for StateTransition {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        // Check for starting state definition
        let start = input.parse::<Token![*]>().is_ok();

        // Parse the DSL
        //
        // Transition DSL:
        // SrcState(OptionalType1) + Event(OptionalType2) [ guard ] / action =
        // DstState(OptionalType3)

        // Input State
        let in_state: Ident = input.parse()?;

        // Possible type on the input state
        let in_state_data_type = if input.peek(token::Paren) {
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

        // Possible type on the input state
        let out_state_data_type = if input.peek(token::Paren) {
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
                        "This is an unsupported type for states.",
                    ))
                }
            }

            Some(input)
        } else {
            None
        };

        Ok(StateTransition {
            start,
            in_state,
            in_state_data_type,
            event,
            event_data_type,
            guard,
            action,
            out_state,
            out_state_data_type,
        })
    }
}

impl parse::Parse for StateMachine {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let mut statemachine = StateMachine::new();

        loop {
            // If the last line ends with a comma this is true
            if input.is_empty() {
                break;
            }

            match input.parse::<Ident>()?.to_string().as_str() {
                "transitions" => {
                    input.parse::<Token![:]>()?;
                    if input.peek(token::Brace) {
                        let content;
                        braced!(content in input);
                        loop {
                            if content.is_empty() {
                                break;
                            }

                            let transition: StateTransition = content.parse()?;
                            statemachine.add_transition(transition);

                            // No comma at end of line, no more transitions
                            if content.is_empty() {
                                break;
                            }

                            if let Err(_) = content.parse::<Token![,]>() {
                                break;
                            };
                        }
                    }
                }
                "guard_error" => {
                    input.parse::<Token![:]>()?;
                    let guard_error: Type = input.parse()?;

                    // Check so the type is supported
                    match &guard_error {
                        Type::Array(_)
                        | Type::Path(_)
                        | Type::Ptr(_)
                        | Type::Reference(_)
                        | Type::Slice(_)
                        | Type::Tuple(_) => (),
                        _ => {
                            return Err(parse::Error::new(
                                guard_error.span(),
                                "This is an unsupported type for guard error.",
                            ))
                        }
                    }

                    statemachine.guard_error = Some(guard_error);
                }
                "temporary_context" => {
                    input.parse::<Token![:]>()?;
                    let temporary_context_type: Type = input.parse()?;

                    // Check so the type is supported
                    match &temporary_context_type {
                        Type::Array(_)
                        | Type::Path(_)
                        | Type::Ptr(_)
                        | Type::Reference(_)
                        | Type::Slice(_)
                        | Type::Tuple(_) => (),
                        _ => {
                            return Err(parse::Error::new(
                                temporary_context_type.span(),
                                "This is an unsupported type for the temporary state.",
                            ))
                        }
                    }

                    // Store the temporary context type
                    statemachine.temporary_context_type = Some(temporary_context_type);

                }
                keyword => {
                    return Err(parse::Error::new(
                        input.span(),
                        format!("Unknown keyword {}. Support keywords: [\"transitions\", \"temporary_context\", \"guard_error\"]", keyword)
                    ))
                }
            }

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
