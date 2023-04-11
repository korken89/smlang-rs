//! An example of using state data to propagate events (See issue-17)

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    name: OnEntryExample,
    transitions: {
        *D0 + ToD1 = D1,
        D1 + ToD2 = D2,
    },
    generate_entry_exit_states: true,
}

/// Context
pub struct Context {
    exited_d0: bool,
    entered_d1: bool,
}

impl OnEntryExampleStateMachineContext for Context {
    fn on_exit_d0(&mut self) {
        self.exited_d0 = true;
    }
    fn on_entry_d1(&mut self) {
        self.entered_d1 = true;
    }
}

fn main() {
    let mut sm = OnEntryExampleStateMachine::new(Context {
        exited_d0: false,
        entered_d1: false,
    });

    // first event starts the dominos
    let _ = sm.process_event(OnEntryExampleEvents::ToD1).unwrap();

    assert!(matches!(sm.state(), Ok(&OnEntryExampleStates::D1)));
    assert!(sm.context().exited_d0);
    assert!(sm.context().entered_d1);
}
