use super::transition::{StateTransition, StateTransitions};
use syn::{braced, parse, spanned::Spanned, token, Attribute, Ident, Token, Type};

#[derive(Debug)]
pub struct StateMachine {
    pub temporary_context_type: Option<Type>,
    pub custom_error: bool,
    pub transitions: Vec<StateTransition>,
    pub name: Option<Ident>,
    pub states_attr: Vec<Attribute>,
    pub events_attr: Vec<Attribute>,
}

impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            temporary_context_type: None,
            custom_error: false,
            transitions: Vec::new(),
            name: None,
            states_attr: Vec::new(),
            events_attr: Vec::new(),
        }
    }

    pub fn add_transitions(&mut self, transitions: StateTransitions) {
        for in_state in transitions.in_states {
            let transition = StateTransition {
                in_state,
                event: transitions.event.clone(),
                guard: transitions.guard.clone(),
                action: transitions.action.clone(),
                out_state: transitions.out_state.clone(),
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

                            if content.parse::<Token![,]>().is_err() {
                                break;
                            };
                        }
                    }
                }
                "custom_error" => {
                    input.parse::<Token![:]>()?;
                    let custom_error: syn::LitBool = input.parse()?;
                    if custom_error.value {
                        statemachine.custom_error = true
                    }
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
                "name" => {
                    input.parse::<Token![:]>()?;
                    statemachine.name = Some(input.parse::<Ident>()?);
                }

                "states_attr" => {
                    input.parse::<Token![:]>()?;
                    statemachine.states_attr = Attribute::parse_outer(&input)?;
                }

                "events_attr" => {
                    input.parse::<Token![:]>()?;
                    statemachine.events_attr = Attribute::parse_outer(&input)?;
                }

                keyword => {
                    return Err(parse::Error::new(
                        input.span(),
                        format!(
                            "Unknown keyword {}. Support keywords: [\"name\", \
                                \"transitions\", \
                                \"temporary_context\", \
                                \"custom_error\", \
                                \"states_attr\", \
                                \"events_attr\"
                                ]",
                            keyword
                        ),
                    ))
                }
            }

            // No comma at end of line, no more transitions
            if input.is_empty() {
                break;
            }

            if input.parse::<Token![,]>().is_err() {
                break;
            };
        }

        Ok(statemachine)
    }
}
