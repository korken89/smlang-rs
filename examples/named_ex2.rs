//! Looping state machine
//!
//! An example of a state machine which will loop between State 2 and State 3.
//! A picture depicting the state machine can be found in the README.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    name: Looping,
    transitions: {
        *State1 + Event1 = State2,
        State2 + Event2 = State3,
        State3 + Event3 = State2,
    }
}

/// Context
pub struct Context;

impl LoopingStateMachineContext for Context {}

fn main() {
    let mut sm = LoopingStateMachine::new(Context);
    assert!(matches!(sm.state(), &LoopingStates::State1));

    let r = sm.process_event(LoopingEvents::Event1);
    assert!(matches!(r, Ok(&LoopingStates::State2)));

    let r = sm.process_event(LoopingEvents::Event2);
    assert!(matches!(r, Ok(&LoopingStates::State3)));

    // Go back in the loop a few time
    let r = sm.process_event(LoopingEvents::Event3);
    assert!(matches!(r, Ok(&LoopingStates::State2)));

    let r = sm.process_event(LoopingEvents::Event2);
    assert!(matches!(r, Ok(&LoopingStates::State3)));

    let r = sm.process_event(LoopingEvents::Event3);
    assert!(matches!(r, Ok(&LoopingStates::State2)));

    // Now we cannot use Event1 again, as it is outside the state machine loop
    let r = sm.process_event(LoopingEvents::Event1);
    assert!(matches!(r, Err(LoopingError::InvalidEvent)));
    assert!(matches!(sm.state(), &LoopingStates::State2));
}
