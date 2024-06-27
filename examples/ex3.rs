//! Looping state machine
//!
//! An example of using guards and actions.
//! A picture depicting the state machine can be found in the README.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 [guard] / action1 = State2,
        State2 + Event2 [guard_fail] / action2 = State3,
    }
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn guard(&self) -> Result<bool, ()> {
        // Always ok
        Ok(true)
    }

    fn guard_fail(&self) -> Result<bool, ()> {
        // Always fail
        Ok(false)
    }

    fn action1(&mut self) -> Result<(), ()> {
        //println!("Action 1");
        Ok(())
    }

    fn action2(&mut self) -> Result<(), ()> {
        //println!("Action 1");
        Ok(())
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    assert!(matches!(sm.state(), &States::State1));

    println!("Before action 1");

    // Go through the first guard and action
    let r = sm.process_event(Events::Event1);
    assert!(matches!(r, Ok(&States::State2)));

    println!("After action 1");

    println!("Before action 2");

    // The action will never run as the guard will fail
    let r = sm.process_event(Events::Event2);
    assert!(matches!(r, Err(Error::TransitionsFailed)));

    println!("After action 2");

    // Now we are stuck due to the guard never returning true
    assert!(matches!(sm.state(), &States::State2));
}
