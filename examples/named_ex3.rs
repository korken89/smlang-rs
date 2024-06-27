//! Looping state machine
//!
//! An example of using guards and actions.
//! A picture depicting the state machine can be found in the README.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    name: LoopingWithGuards,
    transitions: {
        *State1 + Event1 [guard] / action1 = State2,
        State2 + Event2 [guard_fail] / action2 = State3,
    }
}

/// Context
pub struct Context;

impl LoopingWithGuardsStateMachineContext for Context {
    fn guard(&mut self) -> Result<bool, ()> {
        // Always ok
        Ok(true)
    }

    fn guard_fail(&mut self) -> Result<bool, ()> {
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
    let mut sm = LoopingWithGuardsStateMachine::new(Context);
    assert!(matches!(sm.state(), &LoopingWithGuardsStates::State1));

    println!("Before action 1");

    // Go through the first guard and action
    let r = sm.process_event(LoopingWithGuardsEvents::Event1);
    assert!(matches!(r, Ok(&LoopingWithGuardsStates::State2)));

    println!("After action 1");

    println!("Before action 2");

    // The action will never run as the guard will fail
    let r = sm.process_event(LoopingWithGuardsEvents::Event2);
    assert!(matches!(r, Err(LoopingWithGuardsError::TransitionsFailed)));

    println!("After action 2");

    // Now we are stuck due to the guard never returning true
    assert!(matches!(sm.state(), &LoopingWithGuardsStates::State2));
}
