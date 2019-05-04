use super::statemachine;

fn guard1() -> bool {
    println!("Guard 1 ok");

    true
}

fn guard2() -> bool {
    println!("Guard 2 ok");

    true
}

fn guard_fail() -> bool {
    false
}

fn action1() {
    println!("Running Action 1");
}

fn action2() {
    println!("Running Action 2");
}

statemachine!(
    *State1 + Event1[guard1] / action1 = State2,
    State2 + Event2[guard2] / action2 = State3,
    State2 + Event3 = State1,
    State3 + Event3 = State1,
    State2 + Event4[guard_fail] = State1,
);

#[test]
fn starting_state() {
    let sm = StateMachine::new();
    assert_eq!(sm.state(), States::State1);
}

#[test]
fn transitions() {
    let mut sm = StateMachine::new();

    let _ = sm.process_event(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event1);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event1);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.process_event(Events::Event3);
    assert_eq!(sm.state(), States::State1);

    let _ = sm.process_event(Events::Event2);
    assert_eq!(sm.state(), States::State1);

    let _ = sm.process_event(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.process_event(Events::Event2);
    assert_eq!(sm.state(), States::State3);
}

#[test]
fn event_error() {
    let mut sm = StateMachine::new();
    assert_eq!(sm.state(), States::State1);

    let output = sm.process_event(Events::Event3);
    assert_eq!(output, Err(Error::InvalidEvent));

    let output = sm.process_event(Events::Event1);
    assert_eq!(output, Ok(States::State2));
}

#[test]
fn guard_error() {
    let mut sm = StateMachine::new();

    let _ = sm.process_event(Events::Event1);
    let output = sm.process_event(Events::Event4);
    assert_eq!(output, Err(Error::GuardFailed));
}
