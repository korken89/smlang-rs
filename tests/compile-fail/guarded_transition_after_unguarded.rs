extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        State1 + Event1 = Fault,
        *State1 + Event1 [guard] = State2,
    }
}

fn main() {}
