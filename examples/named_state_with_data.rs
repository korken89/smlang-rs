//! State data example
//!
//! An example of using state data together with an action.

#![deny(missing_docs)]

use smlang::statemachine;

/// State data
#[derive(PartialEq)]
pub struct MyStateData(pub u32);

statemachine! {
    name: StatesWithData,
    transitions: {
        *State1 + Event1 / action = State2,
        State2(MyStateData) + Event2 = State1,
        // ...
    }
}

/// Context
pub struct Context;

impl StatesWithDataStateMachineContext for Context {
    fn action(&mut self) -> MyStateData {
        MyStateData(42)
    }
}

fn main() {
    let mut sm = StatesWithDataStateMachine::new(Context);
    let result = sm.process_event(StatesWithDataEvents::Event1);

    assert!(matches!(result, Ok(&StatesWithDataStates::State2(MyStateData(42)))));
}
