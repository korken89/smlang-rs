extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State1 + Event1 = State3, //~ State and event combination specified multiple times, remove duplicates.
    }
}

fn main() {}
