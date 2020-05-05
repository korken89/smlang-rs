//! Reference types in events
//!
//! A simple example of a state machine which will get events that contain references.

use smlang::statemachine;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MyReferenceWrapper<'a>(pub &'a u32);

statemachine! {
    *State1 + Event1(&'a [u8]) [guard1] / action1 = State2,
    State2 + Event2(MyReferenceWrapper<'b>) [guard2] / action2 = State3,
}

#[derive(Debug)]
pub struct Context;

impl StateMachineContext for Context {
    fn guard1(&self, event_data: &[u8]) -> bool {
        true
    }

    fn action1(&mut self, event_data: &[u8]) {
    }

    fn guard2(&self, event_data: &MyReferenceWrapper) -> bool {
        true
    }

    fn action2(&mut self, event_data: &MyReferenceWrapper) {
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    assert_eq!(sm.state(), States::State1);
}
