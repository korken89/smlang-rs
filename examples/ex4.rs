//! Linear state machine with data dependent guards and actions
//!
//! A example of a state machine which has values associated to the events.
//! A picture depicting the state machine can be found in the README.

use smlang::statemachine;

statemachine! {
    *State1 + Event1(u32) [guard] / action = State2,
    State2 + Event2((u32, u32, u32)) / action = State3,
}

#[derive(Debug)]
pub struct Context;

impl StateMachineContext for Context {
    fn guard(&self, event: &Events) -> bool {
        event == &Events::Event1(1)
    }

    fn action(&mut self, event: &Events) {
        println!("Action {:?}", event);
    }
}

fn main() {
    let mut sm = StateMachine::<Context>::new(Context);
    assert_eq!(sm.state(), States::State1);

    // Wrong value, guard will fail
    let r = sm.process_event(Events::Event1(2));
    assert_eq!(r, Err(Error::GuardFailed));

    // Now guard will be ok
    let r = sm.process_event(Events::Event1(1));
    assert_eq!(r, Ok(States::State2));

    let r = sm.process_event(Events::Event2((1, 2, 3)));
    assert_eq!(r, Ok(States::State3));
}
