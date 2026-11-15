extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 / action = State2, //~ The starting state cannot have eventless transitions, because `new()` cannot run guards or actions
    }
}

fn main() {}
