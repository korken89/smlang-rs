//! State data example
//!
//! An example of defining internal transitions combined with wildcard input states.

#![deny(missing_docs)]
use smlang::{assert_transition, statemachine};

statemachine! {
    transitions: {
        *State1 + Event2 = State2,
        State2 + Event3 = State3,
        _ + Event1 / increment_count,      // Internal transition (implicit: omitting target state)
        _ + Event3 / increment_count = _ , // Internal transition (explicit: using _ as target state)
    },
    derive_states: [Debug, Clone,  Copy]
}
/// Context
#[derive(Debug)]
pub struct Context {
    count: u32,
}
impl StateMachineContext for Context {
    fn increment_count(&mut self) -> Result<(), ()> {
        self.count += 1;
        Ok(())
    }
}
fn main() {
    let mut sm = StateMachine::new(Context { count: 0 });

    assert_transition!(sm, Events::Event1, States::State1, 1);
    assert_transition!(sm, Events::Event2, States::State2, 1);
    assert_transition!(sm, Events::Event3, States::State3, 1);
    assert_transition!(sm, Events::Event1, States::State3, 2);
    assert_transition!(sm, Events::Event3, States::State3, 3);

    assert!(sm.process_event(Events::Event2).is_err()); // InvalidEvent
    assert_eq!(States::State3, sm.state);
}
