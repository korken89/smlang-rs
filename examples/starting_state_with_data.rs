//! State data example
//!
//! An example of using state data together with an action.

#![deny(missing_docs)]

use smlang::statemachine;

/// State data
#[derive(PartialEq)]
pub struct MyStateData(pub u32);

statemachine! {
    transitions: {
        State2 + Event2 / action = State1,
        *State1(MyStateData) + Event1 = State2,
        // ...
    }
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn action(&mut self) -> MyStateData {
        MyStateData(42)
    }
}

fn main() {
    let mut sm = StateMachine::new(Context, MyStateData(42));
    let _ = sm.process_event(Events::Event1);
    let result = sm.process_event(Events::Event2);

    assert!(result == Ok(&States::State1(MyStateData(42))));
}
