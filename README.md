# smlang: A `no_std` State Machine Language DSL in Rust

![Build Status](https://github.com/korken89/smlang-rs/actions/workflows/build.yml/badge.svg)
![Documentation](https://github.com/korken89/smlang-rs/actions/workflows/docs.yml/badge.svg)

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

The DSL supports wildcards and pattern matching for input states similar to rust pattern matching:

```rust
statemachine!{
    transitions: {
        *State1 | State3 + ToState2 = State2,
        State1 | State2 + ToState3 = State3,
        _ + ToState4 = State4,
        State4 + ToState1 = State1,
    }
    // ...
}
```

Which is equivalent to:

```rust
statemachine!{
    transitions: {
        *State1 + ToState2 = State2,
        State3 + ToState2 = State2,

        State1 + ToState3 = State3,
        State2 + ToState3 = State3,

        State1 + ToState4 = State4,
        State2 + ToState4 = State4,
        State3 + ToState4 = State4,
        State4 + ToState4 = State4,

        State4 + ToState1 = State1,
    }
    // ...
}
```
See example `examples/input_state_pattern_match.rs` for a usage example.

#### Guard expressions

Guard expression in square brackets [] allows to define a boolean expressions of multiple guard functions.
For example:
```rust
statemachine! {
  transitions: {
      *Init + Login(Entry) [valid_entry] / attempt = LoggedIn,
      Init + Login(Entry) [!valid_entry && !too_many_attempts] / attempt = Init,
      Init + Login(Entry) [!valid_entry && too_many_attempts] / attempt = LoginDenied,
      LoggedIn + Logout / reset = Init,
  }
}
```
Guard expressions may consist of guard function names, and their combinations with &&, || and ! operations.

#### Multiple guarded transitions for the same state and triggering event
Multiple guarded transitions for the same state and triggering event are supported (see the example above).
It is assumed that only one guard is enabled in such a case to avoid a conflict over which transition should be selected.
However, if there is a conflict and more than one guard is enabled, the first enabled transition,
in the order they appear in the state machine definition, will be selected.

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

Any state may have some data associated with it:

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

If the starting state contains data, this data must be provided after the context when creating a new machine.

```rust
pub struct MyStateData(pub u32);

statemachine!{
    transitions: {
        State2 + Event2 / action = State1(MyStateData),
        *State1(MyStateData) + Event1 = State2,
        // ...
    }
    // ...
}

// ...

let mut sm = StateMachine::new(Context, MyStateData(42));
```

State data may also have associated lifetimes which the `statemachine!` macro will pick up and add the `States` enum and `StateMachine` structure. This means the following will also work:

```rust
pub struct MyStateData<'a>(&'a u32);

statemachine! {
    transitions: {
        *State1 + Event1 / action = State2,
        State2(MyStateData<'a>) + Event2 = State1,
        // ...
    }
    // ...
}
```

See example `examples/state_with_reference_data.rs` for a usage example.

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

Event data may also have associated lifetimes which the `statemachine!` macro will pick up and add the `Events` enum. This means the following will also work:

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

### Async Guard and Action

See example `examples/async.rs` for a usage-example.

## State Machine Examples

Here are some examples of state machines converted from UML to the State Machine Language DSL. Runnable versions of each example is available in the `examples` folder.
The `.png`s are generated with the `graphviz` feature.

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

### Using entry and exit functions in transitions

DSL implementation:

```rust
statemachine!{
    transitions: {
        *State1 + Event1 = State2,
        State2 < exit_state_2 + Event2 = State1,
        State1 > enter_state_3 + Event3 = State3,
        State2 + Event3 = State3,
    }
}
```
For all transitions entering State3, the function `enter_state_3` will be
called. For all transitions exiting State2, the function `exit_state_2` will be
called, in the right order, so first the `exit` function prior to the `entry`
function.

An example is available in `on_entry_on_exit`.

There is also a generic flag available, `generate_on_entry_on_exit`, which will
generate for all states in the statemachine an entry and an exit function. If
they are not used, they will be optimized away by the compiler. An example be
found in `on_entry_on_exit_generic`.

## Helpers

### Auto-derive certain traits for states and events

Setting `derive_events` and `derive_states` fields to an array of traits adds a derive expression to `Events` and `States` enums respectively. To derive Display, use `derive_more::Display`.


```rust
use core::Debug;
use derive_more::Display;
// ...
statemachine!{
    derive_states: [Debug, Display],
    derive_events: [Debug, Display],
    transitions: {
        *State1 + Event1 = State2,
    }
}

// ...

println!("Current state: {}", sm.state().unwrap());
println!("Expected state: {}", States::State1);
println!("Sending event: {}", Events::Event1);

// ...

```

### Hooks for logging events, guards, actions, and state transitions

The `StateMachineContext` trait defines (and provides default, no-op implementations for) functions that are called for each event, guard, action, and state transition. You can provide your
own implementations which plug into your preferred logging mechanism.

```rust
fn log_process_event(&self, current_state: &States, event: &Events) {}
fn log_guard(&self, guard: &'static str, result: &Result<(), ()>) {}
fn log_action(&self, action: &'static str) {}
fn log_state_change(&self, new_state: &States) {}
```

See `examples/state_machine_logger.rs` for an example which uses `derive_states` and `derive_events` to derive `Debug` implementations for easy logging.

## Contributors

List of contributors in alphabetical order:

* Emil Fresk ([@korken89](https://github.com/korken89))
* Mathias Koch ([@MathiasKoch](https://github.com/MathiasKoch))
* Donny Zimmanck ([@dzimmanck](https://github.com/dzimmanck))

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

