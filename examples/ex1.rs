//! Linear state machine
//!
//! A simple example of a state machine which will get stuck in the final state.
//! A picture depicting the state machine can be found in the README.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State2 + Event2 = State3,
    },
}

/// Context
pub struct Context;

impl StateMachineContext for Context {}

fn main() {
    let mut sm = StateMachine::new(Context);
    assert!(matches!(sm.state(), Ok(&States::State1)));

    let r = sm.process_event(Events::Event1);
    assert!(matches!(r, Ok(&States::State2)));

    let r = sm.process_event(Events::Event2);
    assert!(matches!(r, Ok(&States::State3)));

    // Now all events will not give any change of state
    let r = sm.process_event(Events::Event1);
    assert!(matches!(r, Err(Error::InvalidEvent)));
    assert!(matches!(sm.state(), Ok(&States::State3)));

    let r = sm.process_event(Events::Event2);
    assert!(matches!(r, Err(Error::InvalidEvent)));
    assert!(matches!(sm.state(), Ok(&States::State3)));
}
