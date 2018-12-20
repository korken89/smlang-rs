extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use syn::parse::{Parse, ParseStream, Result};
use syn::{bracketed, token};
use syn::{parse_macro_input, Ident, Token};

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
struct ParsedStateMachine {
    pub states: HashMap<String, Ident>,
    pub events: HashMap<String, Ident>,
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

        for transition in sm.transitions.iter() {
            // Collect states
            states.insert(transition.in_state.to_string(), transition.in_state.clone());
            states.insert(
                transition.out_state.to_string(),
                transition.out_state.clone(),
            );

            // Collect events
            events.insert(transition.event.to_string(), transition.event.clone());
        }

        ParsedStateMachine { states, events }
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

    // Get only the unique states
    let states = sm.states.iter().map(|(_, value)| value);

    // Extract events
    let events = sm.events.iter().map(|(_, value)| value);

    // Build the output, possibly using quasi-quotation
    let output: proc_macro2::TokenStream = {
        quote! {
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            enum States { #(#states,)* }

            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            enum Events { #(#events,)* }
        }
    };

    // Hand the output tokens back to the compiler
    output.into()
}
