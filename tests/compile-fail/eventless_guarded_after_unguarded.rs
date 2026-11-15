extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 = State2,
        State2 = State3,
        State2 [guard] = State4, //~ State2 (eventless): [guard()] : guarded transition is unreachable because it follows an unguarded transition, which handles all cases
    }
}

fn main() {}
