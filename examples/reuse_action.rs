//! Reuse the same aciton more than once
//!
//! This example shows how to use the same action in multiple transitions.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 / action = State2,
        State1 + Event2 / action = State3,
        State2 + Event2 = State1,
    }
}

/// Action will increment our context
pub struct Context(usize);

impl StateMachineContext for Context {
    fn action(&mut self) {
        self.0 += 1;
    }
}

fn main() {
    let mut sm = StateMachine::new(Context(0));
    assert!(sm.state() == &States::State1);
    assert!(sm.context.0 == 0);

    // triggers action
    let r = sm.process_event(Events::Event1);
    assert!(r == Ok(&States::State2));
    assert!(sm.context.0 == 1);

    let r = sm.process_event(Events::Event2);
    assert!(r == Ok(&States::State1));
    assert!(sm.context.0 == 1);

    // triggers the same action
    let r = sm.process_event(Events::Event2);
    assert!(r == Ok(&States::State3));
    assert!(sm.context.0 == 2);
}
