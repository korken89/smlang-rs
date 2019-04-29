use sml::statemachine;

fn guard1() -> bool {
    println!("Guard 1 ok");

    true
}

fn guard2() -> bool {
    println!("Guard 2 ok");

    true
}

fn action1() {
    println!("Running Action 1");
}

fn action2() {
    println!("Running Action 2");
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
);

fn main() {
    let mut sm = StateMachine::new();
    // assert_eq!(sm.state(), States::State1);

    let _ = sm.run(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event2);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event1);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event1);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event2);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event2);
    // assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event3);
    // assert_eq!(sm.state(), States::State1);

    let _ = sm.run(Events::Event2);
    // assert_eq!(sm.state(), States::State1);

    let _ = sm.run(Events::Event1);
    // assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event2);
    // assert_eq!(sm.state(), States::State3);
}
