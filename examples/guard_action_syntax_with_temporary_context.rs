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

statemachine! {
    temporary_context: &mut u16,
    transitions: {
        *State1 + Event1(MyEventData) [guard1] / action1 = State2,
        State2(MyStateData) + Event2  [guard2] / action2 = State3,
        // ...
    },
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    // Guard1 has access to the data from Event1
    fn guard1(&mut self, temp_context: &mut u16, _event_data: &MyEventData) -> Result<bool, ()> {
        *temp_context += 1;

        Ok(true)
    }

    // Action1 has access to the data from Event1, and need to return the state data for State2
    fn action1(&mut self, temp_context: &mut u16, _event_data: MyEventData) -> MyStateData {
        *temp_context += 1;

        MyStateData(1)
    }

    // Guard2 has access to the data from State2
    fn guard2(&mut self, temp_context: &mut u16, _state_data: &MyStateData) -> Result<bool, ()> {
        *temp_context += 1;

        Ok(true)
    }

    // Action2 has access to the data from State2
    fn action2(&mut self, temp_context: &mut u16, _state_data: MyStateData) {
        *temp_context += 1;
    }
}

fn main() {
    let mut sm = StateMachine::new(Context {});
    let mut val = 0;

    // This invocation will go through 1 guard and one action.
    let r = sm
        .process_event(&mut val, Events::Event1(MyEventData(1)))
        .unwrap();

    assert!(r == &States::State2(MyStateData(1)));
    assert_eq!(val, 2);
}
