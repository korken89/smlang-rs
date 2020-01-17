//! Linear state machine
//!
//! A simple example of a state machine which will get stuck in the final state.
//! A picture depicting the state machine can be found in the README.

use smlang::statemachine;

statemachine! {
    *State1 + Event1 = State2,
    State2 + Event2 = State3,
}

#[derive(Debug, Default)]
pub struct Context;

impl StateMachineContext for Context {}

fn main() {
    let mut sm = StateMachine::<Context>::new();
    assert_eq!(sm.state(), States::State1);

    let r = sm.process_event(Events::Event1);
    assert_eq!(r, Ok(States::State2));

    let r = sm.process_event(Events::Event2);
    assert_eq!(r, Ok(States::State3));

    // Now all events will not give any change of state
    let r = sm.process_event(Events::Event1);
    assert_eq!(r, Err(Error::InvalidEvent));
    assert_eq!(sm.state(), States::State3);

    let r = sm.process_event(Events::Event2);
    assert_eq!(r, Err(Error::InvalidEvent));
    assert_eq!(sm.state(), States::State3);
}
