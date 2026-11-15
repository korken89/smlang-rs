extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1(u8) / action = State2,
        State2 / action = State3, //~ Action `action` can only be reused when all input states, events, and output states have the same data
    }
}

fn main() {}
