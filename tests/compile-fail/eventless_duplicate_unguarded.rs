extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State2 = State3,
        State2 = State4, //~ State2 (eventless): Unguarded eventless transition specified multiple times, remove duplicates.
    }
}

fn main() {}
