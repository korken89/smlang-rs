extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State1 [guard] = State3, //~ The starting state cannot have eventless transitions, because `new()` cannot run guards or actions
    }
}

fn main() {}
