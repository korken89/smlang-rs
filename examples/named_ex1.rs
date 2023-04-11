//! Linear state machine
//!
//! A simple example of a state machine which will get stuck in the final state.
//! A picture depicting the state machine can be found in the README.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    name: Linear,
    derive_states: [Debug],
    transitions: {
        *State1 + Event1 = State2,
        State2 + Event2 = State3,
    },
}

/// Context
pub struct Context;

impl LinearStateMachineContext for Context {}

fn main() {
    let mut sm = LinearStateMachine::new(Context);
    assert!(matches!(sm.state(), Ok(&LinearStates::State1)));

    let r = sm.process_event(LinearEvents::Event1);
    assert!(matches!(r, Ok(&LinearStates::State2)));

    let r = sm.process_event(LinearEvents::Event2);
    assert!(matches!(r, Ok(&LinearStates::State3)));

    // Now all events will not give any change of state
    let r = sm.process_event(LinearEvents::Event1);
    assert!(matches!(r, Err(LinearError::InvalidEvent)));
    assert!(matches!(sm.state(), Ok(&LinearStates::State3)));

    let r = sm.process_event(LinearEvents::Event2);
    assert!(matches!(r, Err(LinearError::InvalidEvent)));
    assert!(matches!(sm.state(), Ok(&LinearStates::State3)));
}