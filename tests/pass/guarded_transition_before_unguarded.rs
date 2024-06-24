extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 [guard] = State2,
        State1 + Event1 = Fault,
    }
}

fn main() {}
