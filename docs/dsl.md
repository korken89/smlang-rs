## Transition DSL

The state machine macro DSL is defined as follows:

```rust
use smlang::statemachine;

statemachine!{
    // [Optional] An optional prefix to name the generated state machine trait code. This
    // can be used to allow multiple state machines to exist in the same source
    // file. The generated trait and types are `<name>States`, `<name>Events`,
    // and `<name>StateMachine` respectively.
    name: Name,

    // [Optional] Can be used if a temporary context is needed within the state machine
    // API. When specified, the temporary context is provided in
    // `StateMachine::process_event()` and is exposed in guards and actions as
    // the second argument.
    temporary_context: u32,

    // [Optional] Can be optionally specified to add a new `type Error` to the
    // generated `StateMachineContext` trait to allow guards to return a custom
    // error type instead of `()`.
    custom_guard_error: false,

    // [Optional] A list of derive names for the generated `States` and `Events`
    // enumerations respectively. For example, to `#[derive(Debug)]`, these
    // would both be specified as `[Debug]`.
    derive_states: [],
    derive_events: [],

    transitions: {
        // * denotes the starting state
        *StartState + Event1 [ guard1] / action1 = DstState1,

        // Guards and actions can be async functions.
        SrcState2 + Event2 [ async guard2 ] / async action2 = DstState2,

        // Pattern matching can be used to support multiple states with the same
        // transition event.
        StartState | SrcState2 + Event3 [ guard3] / action3 = DstState3,

        // ..or wildcarding can be used to allow all states to share a
        // transition event.
        _ + Event4 = DstState4,

        // States can contain data
        StateWithData(u32) + Event = DstState5,
        StateWithOtherData(&'a u32) + Event = DstState5,

        // Guards can be logically combined using `!`, `||`, and `&&`.
        SrcState6 + Event6 [ async guard6 || other_guard6 ] / action6 = DstState6,
        SrcState7 + Event7 [ async guard7 && !other_guard7 ] / action7 = DstState7,
    }
    // ...
}
```
