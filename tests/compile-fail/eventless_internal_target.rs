extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State2 [guard] = _, //~ expected `+ Event`, or an explicit target state (`= State`) for an eventless transition
    }
}

fn main() {}
