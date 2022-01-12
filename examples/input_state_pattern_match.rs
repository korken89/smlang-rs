//! Pattern Matching State Machine
//!
//! This demonstrates the use of input state pattern matching so that states that share a common
//! transition to the same output state can be described more succinctly
#![deny(missing_docs)]

use smlang::statemachine;

// statemachine! {
//     transitions: {
//         *Idle + Charge = Charging,
//         Idle + Discharge = Discharging,
//         Charging + ChargeComplete = Charged,
//         Discharging + DischargeComplete = Discharged,
//         Charged + Discharge = Discharging,
//         Dischaged + Charge = Charging,
//         Charging + Discharge = Discharging,
//         Discharging + Charge = Charging,
//         Idle + FaultDetected = Fault,
//         Charging + FaultDetected = Fault,
//         Discharging + FaultDetected = Fault,
//         Charged + FaultDetected = Fault,
//         Discharged + FaultDetected = Fault,
//         Fault + FaultCleard = Idle,
//     },
// }

// A simple charge/discharge state machine that has a dedicated "Fault" state
statemachine! {
    transitions: {
        *Idle | Discharging | Discharged + Charge = Charging,
        Idle | Charging | Charged + Discharge = Discharging,
        Charging + ChargeComplete = Charged,
        Discharging + DischargeComplete = Discharged,
        _ + FaultDetected = Fault,
        Fault + FaultCleard = Idle,
    },
}

/// Context
pub struct Context;

impl StateMachineContext for Context {}

fn main() {
    let mut sm = StateMachine::new(Context);

    assert!(sm.state() == &States::Idle);

    let r = sm.process_event(Events::Charge);
    assert!(r == Ok(&States::Charging));

    let r = sm.process_event(Events::Discharge);
    assert!(r == Ok(&States::Discharging));

    let r = sm.process_event(Events::Charge);
    assert!(r == Ok(&States::Charging));

    let r = sm.process_event(Events::ChargeComplete);
    assert!(r == Ok(&States::Charged));

    let r = sm.process_event(Events::Charge);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::Charged);

    let r = sm.process_event(Events::Discharge);
    assert!(r == Ok(&States::Discharging));

    let r = sm.process_event(Events::DischargeComplete);
    assert!(r == Ok(&States::Discharged));

    let r = sm.process_event(Events::Discharge);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::Discharged);

    sm = StateMachine::new_with_state(Context, States::Idle);
    let r = sm.process_event(Events::FaultDetected);
    assert!(r == Ok(&States::Fault));

    sm = StateMachine::new_with_state(Context, States::Charging);
    let r = sm.process_event(Events::FaultDetected);
    assert!(r == Ok(&States::Fault));

    sm = StateMachine::new_with_state(Context, States::Charged);
    let r = sm.process_event(Events::FaultDetected);
    assert!(r == Ok(&States::Fault));

    sm = StateMachine::new_with_state(Context, States::Discharging);
    let r = sm.process_event(Events::FaultDetected);
    assert!(r == Ok(&States::Fault));

    sm = StateMachine::new_with_state(Context, States::Discharged);
    let r = sm.process_event(Events::FaultDetected);
    assert!(r == Ok(&States::Fault));

    let r = sm.process_event(Events::Charge);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::Fault);

    let r = sm.process_event(Events::Discharge);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::Fault);

    let r = sm.process_event(Events::ChargeComplete);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::Fault);

    let r = sm.process_event(Events::DischargeComplete);
    assert!(r == Err(Error::InvalidEvent));
    assert!(sm.state() == &States::Fault);
}
