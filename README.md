# sml-rs: State Machine Language DSL in Rust

[![Build Status](https://travis-ci.org/korken89/trustflight_firmware.svg?branch=master)](https://travis-ci.org/korken89/sml-rs)

A state machine language DSL based on the syntax of [Boost-SML](https://boost-experimental.github.io/sml/).

## Aim

The aim of this DSL is to facilitate the use of state machines, as they quite fast can become overly complicated to write and get an overview of.

The DSL is as follows:

```
Transition DSL (from Boost-SML):
src_state + event [ guard ] / action = dst_state

Defining starting state:
*src_state + event [ guard ] / action = dst_state
```

Small example with 3 states, 3 events, 2 guards and 2 actions:

```rust
use sml::statemachine;

fn guard1() -> bool {
    println!("Guard 1 ok");

    true
}

fn guard2() -> bool {
    println!("Guard 2 ok");

    true
}

fn action1() {
    println!("Running Action 1");
}

fn action2() {
    println!("Running Action 2");
}

statemachine!(
    *State1 + Event1[guard1] / action1 = State2,
    State2 + Event2[guard2] / action2 = State3,
    State2 + Event3 = State1,
    State3 + Event3 = State1,
);

fn main() {
    let mut sm = StateMachine::new();
    assert_eq!(sm.state(), States::State1);

    let _ = sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    let _ = sm.run(Events::Event3);
    assert_eq!(sm.state(), States::State1);

    let _ = sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State1);

    let _ = sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    let _ = sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);
}
```


## Contributors

List of contributors in alphabetical order:

* Emil Fresk ([@korken89](https://github.com/korken89))

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

