//! Event data example
//!
//! An example of using event data together with a guard and action.

#![deny(missing_docs)]

use smlang::statemachine;

/// Event data
#[derive(PartialEq)]
pub struct MyEventData(pub u32);

statemachine!{
    *State1 + Event1(MyEventData) [guard] / action = State2,
    // ...
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn guard(&mut self, event_data: &MyEventData) -> bool {
        event_data == &MyEventData(42)
    }

    fn action(&mut self, event_data: &MyEventData) {
        println!("Got valid Event Data = {}", event_data.0);
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    let result = sm.process_event(Events::Event1(MyEventData(1))); // Guard will fail

    assert!(result == Err(Error::GuardFailed));

    let result = sm.process_event(Events::Event1(MyEventData(42))); // Guard will pass

    assert!(result == Ok(&States::State2));

}
