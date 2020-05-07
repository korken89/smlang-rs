//! Context with members example
//!
//! An example of using the context structure with members for counting the number of transitions.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    *State1 + Event1 / count_transition1 = State2,
    State2 + Event2 / count_transition2 = State1,
}

/// Context with member
pub struct Context {
    /// Number of transitions
    pub num_transitions: usize,
}

impl StateMachineContext for Context {
    fn count_transition1(&mut self) {
        self.num_transitions += 1;
    }

    fn count_transition2(&mut self) {
        self.num_transitions += 1;
    }
}

fn main() {
    let mut sm = StateMachine::new(Context { num_transitions: 0 });

    sm.process_event(Events::Event1).ok(); // ++
    sm.process_event(Events::Event1).ok(); // Will fail
    sm.process_event(Events::Event2).ok(); // ++

    assert_eq!(sm.context().num_transitions, 2);

    // ...
}
