//! Mixed eventless and event transition example
//!
//! `StartEvent` runs `configure` and enters `CheckConfiguration`. Its guarded eventless
//! transition is evaluated right after it is entered: if the `configuration_successful` guard
//! passes, the machine moves on to `Ready` within the same `process_event` call.
//!
//! There is no unguarded fallback, so if the guard fails, the machine waits in
//! `CheckConfiguration`. This is not an error. From there, two events are accepted:
//!
//! - `RetryEvent` runs `configure` again as an internal transition. The eventless transition
//!   is evaluated again afterwards, so a fixed configuration moves the machine on to `Ready`.
//! - `OverrideEvent` leaves the check and enters `ManualMode`.
//!
//! The guard is only evaluated after a transition that ends in `CheckConfiguration`. A
//! configuration that becomes valid in the meantime has no effect until `RetryEvent` arrives.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    states_attr: #[derive(Debug)],
    transitions: {
        *Idle + StartEvent / configure = CheckConfiguration,
        CheckConfiguration [ configuration_successful ] = Ready,
        CheckConfiguration + OverrideEvent = ManualMode,
        CheckConfiguration + RetryEvent / configure = _,
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
    // Runs on `StartEvent` and on every `RetryEvent`
    fn configure(&mut self) -> Result<(), ()> {
        self.configuration_valid = self.setting != 0;
        Ok(())
    }

    // Evaluated whenever a transition ends in `CheckConfiguration`, without another event
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

    // An invalid configuration waits in `CheckConfiguration`. After the setting is fixed,
    // `RetryEvent` runs `configure` again and the machine moves on to `Ready`.
    let mut sm = StateMachine::new(Context {
        setting: 0,
        configuration_valid: false,
    });
    let state = sm.process_event(Events::StartEvent).unwrap();
    assert!(matches!(state, States::CheckConfiguration));

    sm.context_mut().setting = 42;
    let state = sm.process_event(Events::RetryEvent).unwrap();
    assert!(matches!(state, States::Ready));

    // `OverrideEvent` leaves a failed check for `ManualMode`
    let mut sm = StateMachine::new(Context {
        setting: 0,
        configuration_valid: false,
    });
    let state = sm.process_event(Events::StartEvent).unwrap();
    assert!(matches!(state, States::CheckConfiguration));

    let state = sm.process_event(Events::OverrideEvent).unwrap();
    assert!(matches!(state, States::ManualMode));

    /* $ cargo run --example eventless_transition_mixed
    Idle -> CheckConfiguration
    CheckConfiguration -> Ready
    Idle -> CheckConfiguration
    CheckConfiguration -> Ready
    Idle -> CheckConfiguration
    CheckConfiguration -> ManualMode
    */
}
