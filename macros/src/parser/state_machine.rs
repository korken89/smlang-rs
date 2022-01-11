use super::transition::{StateTransition, StateTransitions};
use syn::{braced, parse, spanned::Spanned, token, Ident, Token, Type};

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

    pub fn add_transitions(&mut self, transitions: StateTransitions) {
        for in_state in transitions.in_states {
            let transition = StateTransition {
                in_state,
                event: transitions.event.clone(),
                event_data_type: transitions.event_data_type.clone(),
                guard: transitions.guard.clone(),
                action: transitions.action.clone(),
                out_state: transitions.out_state.clone(),
                out_state_data_type: transitions.out_state_data_type.clone(),
            };
            self.transitions.push(transition);
        }
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

                            let transitions: StateTransitions = content.parse()?;
                            statemachine.add_transitions(transitions);

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
