extern crate smlang;

use smlang::statemachine;

statemachine! {
    *State1 + Event1 = State2(u32), //~ This state has data associated, but not action is define here to provide it.
}

fn main() {}

