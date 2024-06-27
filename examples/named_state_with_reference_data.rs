//! State data example
//!
//! An example of using referenced state data with lifetimes together with an action.

#![deny(missing_docs)]

use smlang::statemachine;

/// State data
#[derive(PartialEq)]
pub struct MyStateData<'a>(&'a u32);

statemachine! {
    name: StatesWithRefData,
    transitions: {
        *State1 + Event1 / action = State2,
        State2(MyStateData<'a>) + Event2 = State1,
        // ...
    }
}

/// Context
pub struct Context;

impl StatesWithRefDataStateMachineContext for Context {
    fn action<'a>(&mut self) -> Result<MyStateData<'a>, ()> {
        Ok(MyStateData(&42))
    }
}

fn main() {
    let mut sm = StatesWithRefDataStateMachine::new(Context);
    let result = sm.process_event(StatesWithRefDataEvents::Event1);

    assert!(matches!(
        result,
        Ok(&StatesWithRefDataStates::State2(MyStateData(&42)))
    ));
}
