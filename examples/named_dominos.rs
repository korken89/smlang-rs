//! An example of using state data to propagate events (See issue-17)

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    name: Dominos,
    transitions: {
        *D0 +  ToD1 / to_d2  = D1,
        D1(Option<DominosEvents>) +  ToD2 / to_d3  = D2,
        D2(Option<DominosEvents>) +  ToD3 / to_d4  = D3,
        D3(Option<DominosEvents>) +  ToD4 / to_d5  = D4,
        D4(Option<DominosEvents>) +  ToD5  = D5,
    }
}

/// Context
pub struct Context;

impl DominosStateMachineContext for Context {
    fn to_d2(&mut self) -> Result<Option<DominosEvents>, ()> {
        Ok(Some(DominosEvents::ToD2))
    }

    fn to_d3(&mut self, _state_data: &Option<DominosEvents>) -> Result<Option<DominosEvents>, ()> {
        Ok(Some(DominosEvents::ToD3))
    }

    fn to_d4(&mut self, _state_data: &Option<DominosEvents>) -> Result<Option<DominosEvents>, ()> {
        Ok(Some(DominosEvents::ToD4))
    }

    fn to_d5(&mut self, _state_data: &Option<DominosEvents>) -> Result<Option<DominosEvents>, ()> {
        Ok(Some(DominosEvents::ToD5))
    }
}

// The macros does not derive Copy/Clone traits to the events, so we need to add them so that the
// event can be moved out of the state data
impl Copy for DominosEvents {}
impl Clone for DominosEvents {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {
    let mut sm = DominosStateMachine::new(Context);

    // first event starts the dominos
    let mut event = Some(DominosEvents::ToD1);

    // use a while let loop to let the events propagate and the dominos fall
    while let Some(e) = event {
        let state = sm.process_event(e).unwrap();

        // use pattern matching to extract the event from any state with an action that fires one
        // good practice here NOT to use a wildcard to ensure you don't miss any states
        event = match state {
            DominosStates::D0 => None,
            DominosStates::D1(event) => *event,
            DominosStates::D2(event) => *event,
            DominosStates::D3(event) => *event,
            DominosStates::D4(event) => *event,
            DominosStates::D5 => None,
        };
    }

    // All the dominos fell!
    assert!(matches!(sm.state(), &DominosStates::D5));
}
