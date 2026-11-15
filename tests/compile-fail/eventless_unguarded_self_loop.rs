extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *Start + Event1 = A,
        A / action = A, //~ Unguarded eventless transitions form an infinite loop: A -> A (no guarded eventless transition leads out of it)
    }
}

fn main() {}
