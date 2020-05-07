//! Looping state machine
//!
//! An example of a state machine which will loop between State 2 and State 3.
//! A picture depicting the state machine can be found in the README.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    *State1 + Event1 = State2,
    State2 + Event2 = State3,
    State3 + Event3 = State2,
}

/// Context
pub struct Context;

impl StateMachineContext for Context {}

fn main() {
    let mut sm = StateMachine::new(Context);
    assert!(sm.state() == &States::State1);

    let r = sm.process_event(Events::Event1);
    assert!(r == Ok(&States::State2));

    let r = sm.process_event(Events::Event2);
    assert!(r == Ok(&States::State3));

    // Go back in the loop a few time
    let r = sm.process_event(Events::Event3);
    assert!(r == Ok(&States::State2));

    let r = sm.process_event(Events::Event2);
    assert!(r == Ok(&States::State3));

    let r = sm.process_event(Events::Event3);
    assert!(r == Ok(&States::State2));

    // Now we cannot use Event1 again, as it is outside the state machine loop
    let r = sm.process_event(Events::Event1);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::State2);
}
