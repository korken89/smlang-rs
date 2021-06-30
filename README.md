# smlang: A `no_std` State Machine Language DSL in Rust

[![Build Status](https://travis-ci.org/korken89/smlang-rs.svg?branch=master)](https://travis-ci.org/korken89/smlang-rs)

> A state machine language DSL based on the syntax of [Boost-SML](https://boost-ext.github.io/sml/).

## Aim

The aim of this DSL is to facilitate the use of state machines, as they quite fast can become overly complicated to write and get an overview of.

## Transition DSL

The DSL is defined as follows:

```rust
statemachine!{
    transitions: {
        *SrcState1 + Event1 [ guard1 ] / action1 = DstState2, // * denotes starting state
        SrcState2 + Event2 [ guard2 ] / action2 = DstState1,
    }
    // ...
}
```

Where `guard` and `action` are optional and can be left out. A `guard` is a function which returns `true` if the state transition should happen, and `false`  if the transition should not happen, while `action` are functions that are run during the transition which are guaranteed to finish before entering the new state.

> This implies that any state machine must be written as a list of transitions.

### State machine context

The state machine needs a context to be defined.
The `StateMachineContext` is generated from the `statemachine!` proc-macro and is what implements guards and actions, and data that is available in all states within the state machine and persists between state transitions:

```rust
statemachine!{
    transitions: {
        State1 + Event1 = State2,
    }
    // ...
}

pub struct Context;

impl StateMachineContext for Context {}

fn main() {
    let mut sm = StateMachine::new(Context);

    // ...
}
```

See example `examples/context.rs` for a usage example.

### State data

Any stat may have some data associated with it (except the starting state), which means that this data is only exists while in this state.

```rust
pub struct MyStateData(pub u32);

statemachine!{
    transitions: {
        State1(MyStateData) + Event1 = State2,
    }
    // ...
}
```

See example `examples/state_with_data.rs` for a usage example.

### Event data

Data may be passed along with an event into the `guard` and `action`:

```rust
pub struct MyEventData(pub u32);

statemachine!{
    transitions: {
        State1 + Event1(MyEventData) [guard] = State2,
    }
    // ...
}
```

Event data may also have associated lifetimes which the `statemachine!` macro will pick up and add the `Events` structure. This means the following will also work:

```rust
pub struct MyEventData<'a>(pub &'a u32);

statemachine!{
    transitions: {
        State1 + Event1(MyEventData<'a>) [guard1] = State2,
        State1 + Event2(&'a [u8]) [guard2] = State3,
    }
    // ...
}
```

See example `examples/event_with_data.rs` for a usage example.

### Guard and Action syntax

See example `examples/guard_action_syntax.rs` for a usage-example.

## State Machine Examples

Here are some examples of state machines converted from UML to the State Machine Language DSL. Runnable versions of each example is available in the `examples` folder.

### Linear state machine

![alt text](./docs/sm1.png "")

DSL implementation:

```rust
statemachine!{
    transitions: {
        *State1 + Event1 = State2,
        State2 + Event2 = State3,
    }
}
```

This example is available in `ex1.rs`.

### Looping state machine

![alt text](./docs/sm2.png "")

DSL implementation:

```rust
statemachine!{
    transitions: {
        *State1 + Event1 = State2,
        State2 + Event2 = State3,
        State3 + Event3 = State2,
    }
}
```

This example is available in `ex2.rs`.

### Using guards and actions

![alt text](./docs/sm3.png "")

DSL implementation:

```rust
statemachine!{
    transitions: {
        *State1 + Event1 [guard] / action = State2,
    }
}
```

This example is available in `ex3.rs`.

## Contributors

List of contributors in alphabetical order:

* Emil Fresk ([@korken89](https://github.com/korken89))
* Mathias Koch ([@MathiasKoch](https://github.com/MathiasKoch))

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

