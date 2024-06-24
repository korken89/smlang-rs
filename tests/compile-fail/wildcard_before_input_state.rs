extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        _ + Event1 = Fault, //~ State and event combination specified multiple times, remove duplicates.
        *State1 + Event1 = State2,
    }
}

fn main() {}
