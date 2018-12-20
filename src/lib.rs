extern crate proc_macro;
extern crate syn;

use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    bracketed,
    token::{self, Colon2},
};
use syn::{parse_macro_input, punctuated::Punctuated, Ident, Path, PathSegment, Token};

use std::collections::HashMap;

#[derive(Debug)]
struct StateMachine {
    pub transitions: Vec<StateTransition>,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine {
            transitions: Vec::new(),
        }
    }

    fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
    }
}

#[derive(Debug)]
struct EventMapping {
    pub event: Ident,
    pub guard: Option<Ident>,
    pub action: Option<Ident>,
    pub out_state: Ident,
}

fn eventmapping_to_tokens(em: &EventMapping) {
    let event = &em.event;

    let event = quote! { Events::#event };

    println!("Ev: {:?}", event.to_string());
}

#[derive(Debug)]
struct ParsedStateMachine {
    pub states: HashMap<String, Ident>,
    pub events: HashMap<String, Ident>,
    pub states_events_mapping: HashMap<String, HashMap<String, EventMapping>>,
}

impl ParsedStateMachine {
    fn new(sm: StateMachine) -> Self {
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
struct StateTransition {
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

#[proc_macro]
pub fn statemachine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the syntax into structures
    let input = parse_macro_input!(input as StateMachine);
    let sm = ParsedStateMachine::new(input);
    println!("ParsedStateMachine: {:#?}", sm);

    // Get only the unique states
    let states = sm.states.iter().map(|(_, value)| value);

    // Extract events
    let events = sm.events.iter().map(|(_, value)| value);

    // Build the states and events output
    let mut output = quote! {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum States { #(#states),* }

        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Events { #(#events),* }
    };

    // Build the state machine runner
    let sm = quote! {
    struct StateMachine {
        state: States,
    }

    impl StateMachine {
        pub fn new() -> Self {
            StateMachine {
                state: States::State1,
            }
        }

        pub fn state(&self) -> States {
            self.state
        }

        pub fn run(&mut self, event: Events) {
            // match self.state {
            //     States::State1 => match event {
            //         Events::Event1 => {
            //             println!("State1, Event1"); // Do something real in the future
            //             if guard1() {
            //                 action1();
            //                 self.state = States::State2;
            //             }
            //         }
            //         _ => println!("State1, {:?}, nothing happens", event),
            //     },
            // }
        }
    }
    };

    output.append_all(sm);

    // Hand the output tokens back to the compiler
    output.into()
}
