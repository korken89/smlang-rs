//! An example of using state data to propagate events (See issue-17)

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    name: OnEntryExample,
    generate_entry_exit_states: true,
    transitions: {
        *D0 < exit_d0 + ToD1 = D1,
        D0 + ToD3 = D3,
        D1  + ToD2 = D2,
        D2 + ToD1 = D1,
        D1 + ToD0 = D0,
    },
}

/// Context
pub struct Context {
    exited_d0: i32,
    entered_d1: i32,
}

impl OnEntryExampleStateMachineContext for Context {
    fn on_exit_d0(&mut self) {
        self.exited_d0 += 1;
    }
    fn on_entry_d1(&mut self) {
        self.entered_d1 += 1;
    }
}

fn main() {
    let mut sm = OnEntryExampleStateMachine::new(Context {
        exited_d0: 0,
        entered_d1: 0,
    });

    // first event starts the dominos
    let _ = sm.process_event(OnEntryExampleEvents::ToD1).unwrap();

    assert!(matches!(sm.state(), Ok(&OnEntryExampleStates::D1)));
    assert_eq!(sm.context().exited_d0, 1);
    assert_eq!(sm.context().entered_d1, 1);

    let _ = sm.process_event(OnEntryExampleEvents::ToD2).unwrap();

    assert!(matches!(sm.state(), Ok(&OnEntryExampleStates::D2)));
    assert_eq!(sm.context().exited_d0, 1);
    assert_eq!(sm.context().entered_d1, 1);

    let _ = sm.process_event(OnEntryExampleEvents::ToD1).unwrap();

    assert!(matches!(sm.state(), Ok(&OnEntryExampleStates::D1)));
    assert_eq!(sm.context().exited_d0, 1);
    assert_eq!(sm.context().entered_d1, 2);

    let _ = sm.process_event(OnEntryExampleEvents::ToD0).unwrap();

    assert!(matches!(sm.state(), Ok(&OnEntryExampleStates::D0)));
    assert_eq!(sm.context().exited_d0, 1);
    assert_eq!(sm.context().entered_d1, 2);

    let _ = sm.process_event(OnEntryExampleEvents::ToD3).unwrap();

    assert!(matches!(sm.state(), Ok(&OnEntryExampleStates::D3)));
    assert_eq!(sm.context().exited_d0, 2);
    assert_eq!(sm.context().entered_d1, 2);
}
