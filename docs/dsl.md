## Transition DSL

The state machine macro DSL is defined as follows:

```rust
statemachine!{
    // An optional prefix to name the generated state machine trait code. This
    // can be used to allow multiple state machines to exist in the same source
    // file. The generated trait and types are `<name>States`, `<name>Events`,
    // and `<name>StateMachine` respectively.
    name: "",

    // Can be used if a temporary context is needed within the state machine
    // API. When specified, the temporary context is provided in
    // `StateMachine::process_event()` and is exposed in guards and actions as
    // the second argument.
    temporary_context: None,

    // Can be optionally specified to add a new `type GuardError` to the
    // generated `StateMachineContext` trait to allow guards to return a custom
    // error type instead of `()`.
    custom_guard_error: false,

    // An optional list of derive names for the generated `States` and `Events`
    // enumerations respectively. For example, to `#[derive(Debug)]`, these
    // would both be specified as `[Debug]`.
    derive_states: [],
    derive_events: [],

    transitions: {
        // * denotes the starting state
        *SrcState1 + Event1 [ guard1 ] / action1 = DstState2,
        SrcState2 + Event2 [ guard2 ] / action2 = DstState1,

        // Pattern matching can be used to support multiple states with the same
        // transition event.
        SrcState1 | SrcState2 + Event3 [ guard3] / action1 = DstState3,

        // States can contain data
        StateWithData(u32) + Event = DstState,
        StateWithData(&'a u32) + Event = DstState,

        // ..or wildcarding can be used to allow all states to share a
        // transition event.
        _ + Event4 = DstState4,
    }
    // ...
}
```
