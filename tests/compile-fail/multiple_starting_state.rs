extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        //~ More than one starting state defined (indicated with *), remove duplicates.
        *State1 + Event1 = State2,
        *State2 + Event2 = State3,
    }
}

fn main() {}
