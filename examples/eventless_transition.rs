//! Eventless transition example
//!
//! NOTE: THIS EXAMPLE DOES NOT COMPILE YET. smlang does not currently support
//! transitions without an event, so the macro rejects `CheckConfiguration [ ... ] = Ready`.
//! It documents the desired behavior for a future feature and is only built when
//! the `eventless-transitions` feature is enabled.
//!
//! The intended behavior: `StartEvent` runs `configure` and enters `CheckConfiguration`.
//! `CheckConfiguration` has no event of its own. It is a choice point: its eventless
//! transitions are evaluated right after it is entered. If the `configuration_successful`
//! guard passes, the machine moves on to `Ready`. Otherwise the unguarded fallback takes it to
//! `Error`. A single `process_event` call therefore ends in `Ready` or `Error`, never in
//! `CheckConfiguration`.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    states_attr: #[derive(Debug)],
    transitions: {
        *Idle + StartEvent / configure = CheckConfiguration,
        CheckConfiguration [ configuration_successful ] = Ready,
        CheckConfiguration = Error,
    }
}

/// Context
pub struct Context {
    /// Raw setting to apply, e.g. read from a configuration file
    setting: u32,
    /// Set by `configure` when `setting` is valid
    configuration_valid: bool,
}

impl StateMachineContext for Context {
    // Runs on `StartEvent`, before `CheckConfiguration` is entered
    fn configure(&mut self) -> Result<(), ()> {
        self.configuration_valid = self.setting != 0;
        Ok(())
    }

    // Evaluated as soon as `CheckConfiguration` is entered, without another event
    fn configuration_successful(&self) -> Result<bool, ()> {
        Ok(self.configuration_valid)
    }

    fn transition_callback(&self, old_state: &States, new_state: &States) {
        println!("{:?} -> {:?}", old_state, new_state);
    }
}

fn main() {
    // A valid configuration ends in `Ready`
    let mut sm = StateMachine::new(Context {
        setting: 42,
        configuration_valid: false,
    });
    let state = sm.process_event(Events::StartEvent).unwrap();
    assert!(matches!(state, States::Ready));

    // An invalid configuration falls through to `Error`
    let mut sm = StateMachine::new(Context {
        setting: 0,
        configuration_valid: false,
    });
    let state = sm.process_event(Events::StartEvent).unwrap();
    assert!(matches!(state, States::Error));

    /* $ cargo run --example eventless_transition
    Idle -> CheckConfiguration
    CheckConfiguration -> Ready
    Idle -> CheckConfiguration
    CheckConfiguration -> Error
    */
}
