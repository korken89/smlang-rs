extern crate smlang;

use smlang::statemachine;

statemachine! {
    transitions: {
        *S1 + E1 / a = S2(u8),
        S1 + E2 / a = S3(u16), //~ Action `a` can only be reused when all input states, events, and output states have the same data
    }
}

fn main() {}
