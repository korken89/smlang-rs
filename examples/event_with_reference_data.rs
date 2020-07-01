//! Reference types in events
//!
//! A simple example of a state machine which will get events that contain references.

#![deny(missing_docs)]

use smlang::statemachine;

/// Reference wrapper
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MyReferenceWrapper<'a>(pub &'a u32);

statemachine! {
    *State1 + Event1(&'a [u8]) [guard1] / action1 = State2,
    State2 + Event2(MyReferenceWrapper<'b>) [guard2] / action2 = State3,
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    fn guard1(&mut self, event_data: &[u8]) -> bool {
        // Only ok if the slice is not empty
        !event_data.is_empty()
    }

    fn action1(&mut self, event_data: &[u8]) {
        println!("Got valid Event Data = {:?}", event_data);
    }

    fn guard2(&mut self, event_data: &MyReferenceWrapper) -> bool {
        *event_data.0 > 9000
    }

    fn action2(&mut self, event_data: &MyReferenceWrapper) {
        println!("Got valid Event Data = {}", event_data.0);
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);

    let result = sm.process_event(Events::Event1(&[])); // Guard will fail
    assert!(result == Err(Error::GuardFailed));
    let result = sm.process_event(Events::Event1(&[1, 2, 3])); // Guard will pass
    assert!(result == Ok(&States::State2));

    let r = 42;
    let result = sm.process_event(Events::Event2(MyReferenceWrapper(&r))); // Guard will fail
    assert!(result == Err(Error::GuardFailed));

    let r = 9001;
    let result = sm.process_event(Events::Event2(MyReferenceWrapper(&r))); // Guard will pass
    assert!(result == Ok(&States::State3));
}
