//! Event data example
//!
//! An example of using event data together with a guard and action.

#![deny(missing_docs)]

use smlang::statemachine;

/// Event data
#[derive(PartialEq)]
pub struct MyEventData(pub u32);

statemachine! {
    transitions: {
        *State1 + Event1(MyEventData) [guard] / action = State2,
        // ...
    }
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn guard(&self, event_data: &MyEventData) -> Result<bool, ()> {
        Ok(event_data == &MyEventData(42))
    }

    fn action(&mut self, event_data: MyEventData) -> Result<(), ()> {
        println!("Got valid Event Data = {}", event_data.0);
        Ok(())
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    let result = sm.process_event(Events::Event1(MyEventData(1))); // Guard will fail

    assert!(matches!(result, Err(Error::TransitionsFailed)));

    let result = sm.process_event(Events::Event1(MyEventData(42))); // Guard will pass

    assert!(matches!(result, Ok(&States::State2)));
}
