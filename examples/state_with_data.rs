//! State data example
//!
//! An example of using state data together with an action.

#![deny(missing_docs)]

use smlang::statemachine;

/// State data
#[derive(PartialEq)]
pub struct MyStateData(pub u32);

statemachine! {
    *State1 + Event1 / action = State2,
    State2(MyStateData) + Event2 = State1,
    // ...
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn action(&mut self) -> MyStateData {
        MyStateData(42)
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    let result = sm.process_event(Events::Event1);

    assert!(result == Ok(&States::State2(MyStateData(42))));
}
