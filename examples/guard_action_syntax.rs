//! Guard and action syntax example
//!
//! An example of using guards and actions with state and event data.

#![deny(missing_docs)]

use smlang::statemachine;

/// Event data
#[derive(PartialEq)]
pub struct MyEventData(pub u32);

/// State data
#[derive(PartialEq)]
pub struct MyStateData(pub u32);

statemachine!{
    *State1 + Event1(MyEventData) [guard1] / action1 = State2,
    State2(MyStateData) + Event2  [guard2] / action2 = State3,
    // ...
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    // Guard1 has access to the data from Event1
    fn guard1(&mut self, _event_data: &MyEventData) -> bool {
        todo!()
    }

    // Action1 has access to the data from Event1, and need to return the state data for State2
    fn action1(&mut self, _event_data: &MyEventData) -> MyStateData {
        todo!()
    }

    // Guard2 has access to the data from State2
    fn guard2(&mut self, _state_data: &MyStateData) -> bool {
        todo!()
    }

    // Action2 has access to the data from State2
    fn action2(&mut self, _state_data: &MyStateData) {
        todo!()
    }
}

fn main() {

}
