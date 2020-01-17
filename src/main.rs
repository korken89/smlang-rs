use smlang_macros::statemachine;

#[derive(Debug, Default)]
pub struct Context;

impl StateMachineContext for Context {
    fn guard1(&self) -> bool {
        //println!("Guard 1 ok");

        true
    }

    fn guard2(&self) -> bool {
        //println!("Guard 2 ok");

        true
    }

    fn guard_fail(&self) -> bool {
        //println!("Guard 2 ok");

        true
    }

    fn action1(&mut self) {
        //println!("Running Action 1");
    }

    fn action2(&mut self) {
        //println!("Running Action 2");
    }
}

// Transition DSL (from Boost-SML):
// src_state + event [ guard ] / action = dst_state
//
// Defining starting state:
// *src_state + event [ guard ] / action = dst_state
statemachine!(
    *State1 + Event1[guard1] / action1 = State2,
    State2 + Event2[guard2] / action2 = State3,
    State2 + Event3 = State1,
    State3 + Event3 = State1,
    State2 + Event4[guard_fail] = State1,
);

fn main() {
    let mut sm = StateMachine::<Context>::new();
    // assert_eq!(sm.state(), States::State1);

    let _ = sm.process_event(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event2);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event1);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event1);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event2);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event2);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event3);
    // assert_eq!(sm.state(), States::State1);

    let _ = sm.process_event(Events::Event2);
    // assert_eq!(sm.state(), States::State1);

    let _ = sm.process_event(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event2);
    // assert_eq!(sm.state(), States::State3);
}
