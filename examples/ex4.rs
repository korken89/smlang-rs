//! Linear state machine with data dependent guards and actions
//!
//! A example of a state machine which has values associated to the events.
//! A picture depicting the state machine can be found in the README.

use smlang::statemachine;

statemachine! {
    *State1 + Event1(u32) [guard1] / action1 = State2,
    State2(i8) + Event2(i32) [guard2] / action2 = State3,
}

#[derive(Debug)]
pub struct Context;

impl StateMachineContext for Context {
    fn guard1(&self, event_data: &u32) -> bool {
        true
    }

    fn guard2(&self, state_data: &i8, event_data: &i32) -> bool {
        true
    }

    fn action1(&mut self, event_data: &u32) -> i8 {
        1
    }

    fn action2(&mut self, state_data: &i8, event_data: &i32) {

    }
}

fn main() {
    let mut sm = StateMachine::new(Context);
    // assert_eq!(sm.state(), States::State1);

    // // Wrong value, guard will fail
    // let r = sm.process_event(Events::Event1(2));
    // assert_eq!(r, Err(Error::GuardFailed));

    // // Now guard will be ok
    // let r = sm.process_event(Events::Event1(1));
    // assert_eq!(r, Ok(States::State2));
}
