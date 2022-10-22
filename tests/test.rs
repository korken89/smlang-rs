extern crate smlang;

use smlang::statemachine;

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}

#[test]
fn wildcard_after_input_state() {
    statemachine! {
        transitions: {
            *State1 + Event1 = State2,
            _ + Event1 = Fault,
        }
    }

    struct Context;
    impl StateMachineContext for Context {}

    let mut sm = StateMachine::new(Context);

    sm.process_event(Events::Event1).unwrap();
    assert!(sm.state() == &States::State2);

    sm.process_event(Events::Event1).unwrap();
    assert!(sm.state() == &States::Fault);
}
