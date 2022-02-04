use smlang::statemachine;

statemachine! {
    transitions: {
        *Init + Event [guard] / action = State1(u32),

        // This transition is not valid because `guard` would have different input arguments from
        // earlier.
        State1(u32) + Event [guard] / action2 = State2(u32),
    }
}

fn main() {}
