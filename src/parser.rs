use syn::parse::{Parse, ParseStream, Result};
use syn::{bracketed, token};
use syn::{Ident, Token};

use std::collections::HashMap;

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

pub fn eventmapping_to_tokens(em: &EventMapping) -> String {
    let event = &em.event;

    //let event_tokens = quote! { Events::#event };

    event.to_string()
    //println!("Ev: {:?}", event.to_string());
}

#[derive(Debug)]
pub struct ParsedStateMachine {
    pub states: HashMap<String, Ident>,
    pub events: HashMap<String, Ident>,
    pub states_events_mapping: HashMap<String, HashMap<String, EventMapping>>,
}

impl ParsedStateMachine {
    pub fn new(sm: StateMachine) -> Self {
        // Check the initial state definition
        let num_start: u32 = sm
            .transitions
            .iter()
            .map(|sm| if sm.start { 1 } else { 0 })
            .sum();

        if num_start == 0 {
            panic!("No starting state defined, indicate the starting state with a *");
        } else if num_start > 1 {
            panic!("More than one starting state defined, remove duplicates");
        }

        let mut states = HashMap::new();
        let mut events = HashMap::new();
        let mut states_events_mapping = HashMap::<String, HashMap<String, EventMapping>>::new();

        for transition in sm.transitions.iter() {
            // Collect states
            states.insert(transition.in_state.to_string(), transition.in_state.clone());
            states.insert(
                transition.out_state.to_string(),
                transition.out_state.clone(),
            );

            // Collect events
            events.insert(transition.event.to_string(), transition.event.clone());

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

                eventmapping_to_tokens(&mapping);

                p.insert(transition.event.to_string(), mapping);
            } else {
                panic!("State and event combination specified multiple times, remove duplicates");
            }
        }

        ParsedStateMachine {
            states,
            events,
            states_events_mapping,
        }
    }
}

#[derive(Debug)]
pub struct StateTransition {
    start: bool,
    in_state: Ident,
    event: Ident,
    guard: Option<Ident>,
    action: Option<Ident>,
    out_state: Ident,
}

impl Parse for StateMachine {
    fn parse(input: ParseStream) -> Result<Self> {
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
            // Transition DSL: src_state + event [ guard ] / action = dst_state

            // State and event
            let in_state: Ident = input.parse()?;
            input.parse::<Token![+]>()?;
            let event: Ident = input.parse()?;

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
                event,
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
