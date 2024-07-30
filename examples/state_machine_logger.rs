//! State machine logger example
//!
//! An example of using the logging hooks on the `StateMachineContext` trait to automatically log
//! events, guards, actions, and state transitions

#![deny(missing_docs)]

use smlang::statemachine;

/// Event data
#[derive(PartialEq, Debug)]
pub struct MyEventData(pub u32);

/// State data
#[derive(PartialEq, Debug)]
pub struct MyStateData(pub u32);

statemachine! {
    derive_states: [Debug],
    derive_events: [Debug],
    transitions: {
        *State1 + Event1(MyEventData) [guard1] / action1 = State2,
        State2(MyStateData) + Event2  [guard2] / action2 = State3,
        // ...
    }
}

/// Context
pub struct Context;

impl StateMachineContext for Context {
    // Guard1 has access to the data from Event1
    fn guard1(&self, event_data: &MyEventData) -> Result<bool, ()> {
        Ok(event_data.0 % 2 == 0)
    }

    // Action1 has access to the data from Event1, and need to return the state data for State2
    fn action1(&mut self, event_data: MyEventData) -> Result<MyStateData, ()> {
        println!("Creating state data for next state");
        Ok(MyStateData(event_data.0))
    }

    // Guard2 has access to the data from State2
    fn guard2(&self, state_data: &MyStateData) -> Result<bool, ()> {
        Ok(state_data.0 % 2 == 0)
    }

    // Action2 has access to the data from State2
    fn action2(&mut self, state_data: &MyStateData) -> Result<(), ()> {
        println!("Printing state data {:?}", state_data);
        Ok(())
    }

    fn log_process_event(&self, current_state: &States, event: &Events) {
        println!(
            "[StateMachineLogger]\t[{:?}] Processing event {:?}",
            current_state, event
        );
    }

    fn log_guard(&self, guard: &'static str, result: bool) {
        if result {
            println!("[StateMachineLogger]\tEnabled `{}`", guard);
        } else {
            println!("[StateMachineLogger]\tDisabled `{}`", guard);
        }
    }

    fn log_action(&self, action: &'static str) {
        println!("[StateMachineLogger]\tRunning `{}`", action);
    }

    fn transition_callback(&self, old_state: &States, new_state: &States) {
        println!(
            "[StateMachineLogger]\tTransitioning {:?} -> {:?}",
            old_state, new_state
        );
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);

    let events = [
        Events::Event1(MyEventData(1)),
        Events::Event1(MyEventData(0)),
        Events::Event2,
    ];

    for event in events {
        let _ = sm.process_event(event);
    }

    /* $ cargo run --example state_machine_logger
    [StateMachineLogger][State1] Processing event Event1(MyEventData(1))
    [StateMachineLogger]    Failed `guard1`
    [StateMachineLogger][State1] Processing event Event1(MyEventData(0))
    [StateMachineLogger]    Passed `guard1`
    Creating state data for next state
    [StateMachineLogger]    Running `action1`
    [StateMachineLogger]    Transitioning to State2(MyStateData(0))
    [StateMachineLogger][State2(MyStateData(0))] Processing event Event2
    [StateMachineLogger]    Passed `guard2`
    Printing state data MyStateData(0)
    [StateMachineLogger]    Running `action2`
    [StateMachineLogger]    Transitioning to State3
    */
}
