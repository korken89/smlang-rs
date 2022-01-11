extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        _ | State2 + Event2 = State1, //~ Wildcards already include all states, so should not be used with input state patterns.
    }
}

fn main() {}
