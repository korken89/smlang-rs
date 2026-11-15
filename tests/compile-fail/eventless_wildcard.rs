extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        _ [guard] = State1, //~ Eventless transitions cannot use a wildcard input state; add an event (`_ + Event`)
    }
}

fn main() {}
