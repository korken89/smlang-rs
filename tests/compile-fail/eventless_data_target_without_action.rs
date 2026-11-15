extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State2 [guard] = State3(u32), //~ This state has data associated, but not action is define here to provide it.
    }
}

fn main() {}
