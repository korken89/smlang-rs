//! An example of using state data to propagate events (See issue-17)

#![deny(missing_docs)]

use std::rc::Rc;
use std::sync::Mutex;

use smlang::statemachine;

statemachine! {
    derive_states: [Debug, Copy, Clone ],
    transitions: {
        *D0 + ToD1 = D1,
        D1 + ToD2 = D2,
    },
}

/// Context
pub struct Context {
    transition_called: Rc<Mutex<bool>>,
    state_exited: Rc<Mutex<Option<States>>>,
    state_entered: Rc<Mutex<Option<States>>>,
}

impl StateMachineContext for Context {
    fn transition_callback(&self, exit: &States, entry: &States) {
        *self.transition_called.lock().unwrap() = true;
        *self.state_exited.lock().unwrap() = Some(*exit);
        *self.state_entered.lock().unwrap() = Some(*entry);
    }
}

fn main() {
    let mut sm = StateMachine::new(Context {
        transition_called: Rc::new(Mutex::new(false)),
        state_exited: Rc::new(Mutex::new(None)),
        state_entered: Rc::new(Mutex::new(None)),
    });

    // first event starts the dominos
    let _ = sm.process_event(Events::ToD1).unwrap();

    assert!(matches!(sm.state(), &States::D1));
    assert!(*sm.context().transition_called.lock().unwrap());
    assert_eq!(*sm.context().state_exited.lock().unwrap(), Some(States::D0));
    assert_eq!(
        *sm.context().state_entered.lock().unwrap(),
        Some(States::D1)
    );
}
