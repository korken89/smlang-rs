extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *Start + Event1 = A,
        A / action = B, //~ Unguarded eventless transitions form an infinite loop: A -> B -> A (no guarded eventless transition leads out of it)
        B = A,
    }
}

fn main() {}
