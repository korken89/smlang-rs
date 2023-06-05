extern crate smlang;

use smlang::statemachine;

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}

#[test]
fn wildcard_after_input_state() {
    statemachine! {
        transitions: {
            *State1 + Event1 = State2,
            _ + Event1 = Fault,
        }
    }

    struct Context;
    impl StateMachineContext for Context {}

    let mut sm = StateMachine::new(Context);

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), Ok(&States::State2)));

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), Ok(&States::Fault)));
}

#[test]
fn multiple_lifetimes() {
    pub struct X;
    pub struct Y;
    pub struct Z;

    statemachine! {
        transitions: {
            *State1 + Event1(&'a X) [guard1] / action1 = State2(&'a X),
            State2(&'a X) + Event2(&'b Y) [guard2] / action2 = State3((&'a X, &'b Y)),
            State4 + Event(&'c Z) [guard3] / action3 = State5,
        }
    }

    struct Context;

    impl StateMachineContext for Context {
        fn guard1<'a>(&mut self, _event_data: &'a X) -> Result<(), ()> {
            Ok(())
        }

        fn guard2<'a, 'b>(&mut self, _state_data: &'a X, _event_data: &'b Y) -> Result<(), ()> {
            Ok(())
        }

        fn guard3<'c>(&mut self, _event_data: &'c Z) -> Result<(), ()> {
            Ok(())
        }

        fn action1<'a>(&mut self, event_data: &'a X) -> &'a X {
            event_data
        }

        fn action2<'a, 'b>(&mut self, state_data: &'a X, event_data: &'b Y) -> (&'a X, &'b Y) {
            (state_data, event_data)
        }

        fn action3<'c>(&mut self, _event_data: &'c Z) {}
    }

    #[allow(dead_code)]
    struct WrappedStates<'a, 'b>(States<'a, 'b>);

    #[allow(dead_code)]
    struct WrappedEvents<'a, 'b, 'c>(Events<'a, 'b, 'c>);
}

#[test]
fn impl_display_events_states() {
    statemachine! {
        impl_display_events: true,
        impl_display_states: true,
        transitions: {
            *Init + Event = End,
        }
    }

    struct Context;
    impl StateMachineContext for Context {}

    let mut sm = StateMachine::new(Context);
    assert_eq!(format!("{}", sm.state().unwrap()), "Init");

    let event = Events::Event;
    assert_eq!(format!("{}", event), "Event");

    sm.process_event(event).unwrap();
    assert!(matches!(sm.state(), Ok(&States::End)));
    assert_eq!(format!("{}", sm.state().unwrap()), "End");
}


#[test]
fn async_guards_and_actions() {
    use async_trait::async_trait;
    use smol;

    smol::block_on(async {
        statemachine! {
            is_async: true,
            transitions: {
                *State1 + Event1 [async guard1] / async action1 = State2,
                _ + Event1 = Fault,
            }
        }

        struct Context;
        #[async_trait]
        impl StateMachineContext for Context {
            async fn guard1(&mut self) -> Result<(), ()> {
                Ok(())
            }

            async fn action1(&mut self) -> () {
                ()
            }
        }

        let mut sm = StateMachine::new(Context);

        sm.process_event(Events::Event1).await.unwrap();
        assert!(matches!(sm.state(), Ok(&States::State2)));

        sm.process_event(Events::Event1).await.unwrap();
        assert!(matches!(sm.state(), Ok(&States::Fault)));
    });
}
