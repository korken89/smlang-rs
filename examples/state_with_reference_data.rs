//! State data example
//!
//! An example of using referenced state data with lifetimes together with an action.

#![deny(missing_docs)]

use smlang::statemachine;

/// State data
#[derive(PartialEq)]
pub struct MyStateData<'a>(&'a u32);

statemachine! {
    transitions: {
        *State1 + Event1 / action = State2,
        State2(MyStateData<'a>) + Event2 = State1,
        // ...
    }
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn action<'a>(&mut self) -> Result<MyStateData<'a>, ()> {
        Ok(MyStateData(&42))
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    let result = sm.process_event(Events::Event1);

    assert!(matches!(result, Ok(&States::State2(MyStateData(&42)))));
}
