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
    name: Battery,
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

impl BatteryStateMachineContext for Context {}

fn main() {
    let mut sm = BatteryStateMachine::new(Context);

    assert!(matches!(sm.state(), &BatteryStates::Idle));

    let r = sm.process_event(BatteryEvents::Charge);
    assert!(matches!(r, Ok(&BatteryStates::Charging)));

    let r = sm.process_event(BatteryEvents::Discharge);
    assert!(matches!(r, Ok(&BatteryStates::Discharging)));

    let r = sm.process_event(BatteryEvents::Charge);
    assert!(matches!(r, Ok(&BatteryStates::Charging)));

    let r = sm.process_event(BatteryEvents::ChargeComplete);
    assert!(matches!(r, Ok(&BatteryStates::Charged)));

    let r = sm.process_event(BatteryEvents::Charge);
    assert!(matches!(r, Err(BatteryError::InvalidEvent)));
    assert!(matches!(sm.state(), &BatteryStates::Charged));

    let r = sm.process_event(BatteryEvents::Discharge);
    assert!(matches!(r, Ok(&BatteryStates::Discharging)));

    let r = sm.process_event(BatteryEvents::DischargeComplete);
    assert!(matches!(r, Ok(&BatteryStates::Discharged)));

    let r = sm.process_event(BatteryEvents::Discharge);
    assert!(matches!(r, Err(BatteryError::InvalidEvent)));
    assert!(matches!(sm.state(), &BatteryStates::Discharged));

    sm = BatteryStateMachine::new_with_state(Context, BatteryStates::Idle);
    let r = sm.process_event(BatteryEvents::FaultDetected);
    assert!(matches!(r, Ok(&BatteryStates::Fault)));

    sm = BatteryStateMachine::new_with_state(Context, BatteryStates::Charging);
    let r = sm.process_event(BatteryEvents::FaultDetected);
    assert!(matches!(r, Ok(&BatteryStates::Fault)));

    sm = BatteryStateMachine::new_with_state(Context, BatteryStates::Charged);
    let r = sm.process_event(BatteryEvents::FaultDetected);
    assert!(matches!(r, Ok(&BatteryStates::Fault)));

    sm = BatteryStateMachine::new_with_state(Context, BatteryStates::Discharging);
    let r = sm.process_event(BatteryEvents::FaultDetected);
    assert!(matches!(r, Ok(&BatteryStates::Fault)));

    sm = BatteryStateMachine::new_with_state(Context, BatteryStates::Discharged);
    let r = sm.process_event(BatteryEvents::FaultDetected);
    assert!(matches!(r, Ok(&BatteryStates::Fault)));

    let r = sm.process_event(BatteryEvents::Charge);
    assert!(matches!(r, Err(BatteryError::InvalidEvent)));
    assert!(matches!(sm.state(), &BatteryStates::Fault));

    let r = sm.process_event(BatteryEvents::Discharge);
    assert!(matches!(r, Err(BatteryError::InvalidEvent)));
    assert!(matches!(sm.state(), &BatteryStates::Fault));

    let r = sm.process_event(BatteryEvents::ChargeComplete);
    assert!(matches!(r, Err(BatteryError::InvalidEvent)));
    assert!(matches!(sm.state(), &BatteryStates::Fault));

    let r = sm.process_event(BatteryEvents::DischargeComplete);
    assert!(matches!(r, Err(BatteryError::InvalidEvent)));
    assert!(matches!(sm.state(), &BatteryStates::Fault));
}
