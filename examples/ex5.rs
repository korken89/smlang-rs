//! Reference types in events
//!
//! A simple example of a state machine which will get events that contain references.

use smlang::statemachine;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MyReferenceWrapper<'a>(pub &'a u32);

statemachine! {
    *State1 + Event1(&'a [u8]) = State2,
    State2 + Event2(MyReferenceWrapper<'b>) = State3,
}

#[derive(Debug)]
pub struct Context;

impl StateMachineContext for Context {}

fn main() {
    let mut sm = StateMachine::new(Context);
    assert_eq!(sm.state(), States::State1);
}
