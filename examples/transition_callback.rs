//! An example of using state data to propagate events (See issue-17)

#![deny(missing_docs)]

use std::sync::{Arc, Mutex};

use smlang::statemachine;

statemachine! {
    generate_transition_callback: true,
    transitions: {
        *D0 + ToD1 = D1,
        D1 + ToD2 = D2,
    },
}

/// Context
pub struct Context {
    transition_called: Arc<Mutex<bool>>,
}

impl StateMachineContext for Context {
    fn transition_callback(&self, _state: &Option<States>) {
        *self.transition_called.lock().unwrap() = true;
    }
}

fn main() {
    let mut sm = StateMachine::new(Context {
        transition_called: Arc::new(Mutex::new(false)),
    });

    // first event starts the dominos
    let _ = sm.process_event(Events::ToD1).unwrap();

    assert!(matches!(sm.state(), Ok(&States::D1)));
    assert!(*sm.context().transition_called.lock().unwrap());
}
