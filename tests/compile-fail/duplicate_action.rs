use smlang::statemachine;

statemachine! {
    transitions: {
        *Init + Event / action = State1(u32),

        // This transition is not valid because `action` would have different input arguments from
        // earlier.
        State1(u32) + Event / action = State2(u32),
    }
}

fn main() {}
