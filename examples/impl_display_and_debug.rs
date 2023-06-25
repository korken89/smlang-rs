//! Display and Debug example
//!
//! An example showing the usage of impl_display_* and impl_debug_*.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    impl_debug_events: true,
    impl_debug_states: true,
    impl_display_events: true,
    impl_display_states: true,
    impl_debug_state_machine: true,
    transitions: {
        *State1 + Event1(i32) / action1 = State2(i32),
        // ...
    }
}

/// Context
#[derive(Debug)]
pub struct Context;

impl StateMachineContext for Context {
    fn action1(&mut self, event_data:i32) -> i32 {
        event_data * 2
    }
}

fn main() {
    let mut sm = StateMachine::new(Context);

    {
        let state = sm.state().unwrap();
        assert_eq!(format!("{sm:?}"), "StateMachine { state: Some(State1), context: Context }");
        assert_eq!(format!("{state:?}"), "State1");
        assert_eq!(format!("{state}"), "State1");
    }

    {
        let state = sm.process_event(Events::Event1(5)).unwrap();
        assert!(matches!(state, &States::State2(10)));
    }

    {
        let state = sm.state().unwrap();
        assert_eq!(format!("{sm:?}"), "StateMachine { state: Some(State2(10)), context: Context }");
        assert_eq!(format!("{state:?}"), "State2(10)");
        assert_eq!(format!("{state}"), "State2(i32)");
    }
}
