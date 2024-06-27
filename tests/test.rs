extern crate smlang;

use derive_more::Display;

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

    #[allow(dead_code)]
    struct Context;

    impl StateMachineContext for Context {
        fn guard1(&mut self, _event_data: &X) -> Result<bool, ()> {
            Ok(true)
        }

        fn guard2(&mut self, _state_data: &X, _event_data: &Y) -> Result<bool, ()> {
            Ok(true)
        }

        fn guard3(&mut self, _event_data: &Z) -> Result<bool, ()> {
            Ok(true)
        }

        fn action1<'a>(&mut self, event_data: &'a X) -> &'a X {
            event_data
        }

        fn action2<'a, 'b>(&mut self, state_data: &'a X, event_data: &'b Y) -> (&'a X, &'b Y) {
            (state_data, event_data)
        }

        fn action3(&mut self, _event_data: &Z) {}
    }

    #[allow(dead_code)]
    struct WrappedStates<'a, 'b>(States<'a, 'b>);

    #[allow(dead_code)]
    struct WrappedEvents<'a, 'b, 'c>(Events<'a, 'b, 'c>);
}

#[test]
fn derive_display_events_states() {
    statemachine! {
        derive_events: [Debug,Display],
        derive_states: [Debug,Display],
        transitions: {
            *Init + Event = End,
        }
    }

    struct Context;
    impl StateMachineContext for Context {}

    let mut sm = StateMachine::new(Context);
    assert!(matches!(sm.state(), Ok(&States::Init)));

    let event = Events::Event;
    assert_eq!(format!("{}", event), "Event");

    sm.process_event(event).unwrap();
    assert!(matches!(sm.state(), Ok(&States::End)));
}

#[test]
fn named_derive_display_events_states() {
    statemachine! {
        name: SM,
        derive_events: [Debug,Display],
        derive_states: [Debug,Display],
        transitions: {
            *Init + Event = End,
        }
    }

    struct Context;
    impl SMStateMachineContext for Context {}

    let mut sm = SMStateMachine::new(Context);
    assert!(matches!(sm.state(), Ok(&SMStates::Init)));

    let event = SMEvents::Event;
    assert_eq!(format!("{}", event), "Event");

    sm.process_event(event).unwrap();
    assert!(matches!(sm.state(), Ok(&SMStates::End)));
}

#[test]
fn async_guards_and_actions() {
    use smol;

    smol::block_on(async {
        statemachine! {
            transitions: {
                *State1 + Event1 [async guard1] / async action1 = State2,
                _ + Event1 = Fault,
            }
        }

        struct Context;
        #[smlang::async_trait]
        impl StateMachineContext for Context {
            async fn guard1(&mut self) -> Result<bool, ()> {
                Ok(true)
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

#[test]
fn guard_expressions() {
    #[derive(PartialEq, Display)]
    pub struct Entry(pub u32);

    statemachine! {
        derive_states: [Display, Debug],
        transitions: {
            *Init + Login(&'a Entry) [valid_entry] / attempt = LoggedIn,
            Init + Login(&'a Entry) [!valid_entry && !too_many_attempts] / attempt = Init,
            Init + Login(&'a Entry) [!valid_entry && too_many_attempts] / attempt = LoginDenied,
            LoggedIn + Logout / reset = Init,
        }
    }

    /// Context
    pub struct Context {
        password: u32,
        attempts: u32,
    }
    impl StateMachineContext for Context {
        fn valid_entry(&mut self, e: &Entry) -> Result<bool, ()> {
            Ok(e.0 == self.password)
        }
        fn too_many_attempts(&mut self, _e: &Entry) -> Result<bool, ()> {
            Ok(self.attempts >= 3)
        }
        fn reset(&mut self) {
            self.attempts = 0;
        }
        fn attempt(&mut self, _e: &Entry) {
            self.attempts += 1;
        }
    }

    let mut sm = StateMachine::new(Context {
        password: 42,
        attempts: 0,
    });
    assert!(matches!(sm.state(), Ok(&States::Init)));

    let bad_entry = Entry(10);
    let good_entry = Entry(42);

    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 1);
    assert!(matches!(sm.state(), Ok(&States::Init)));

    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 2);
    assert!(matches!(sm.state(), Ok(&States::Init)));

    let _ = sm.process_event(Events::Login(&good_entry));
    assert_eq!(sm.context().attempts, 3);
    assert!(matches!(sm.state(), Ok(&States::LoggedIn)));

    let _ = sm.process_event(Events::Logout);
    assert_eq!(sm.context().attempts, 0);
    assert!(matches!(sm.state(), Ok(&States::Init)));

    let _ = sm.process_event(Events::Login(&bad_entry));
    let _ = sm.process_event(Events::Login(&bad_entry));
    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 3);
    assert!(matches!(sm.state(), Ok(&States::Init)));

    // exhausted attempts
    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 4);
    assert!(matches!(sm.state(), Ok(&States::LoginDenied)));

    // Invalid event, as we are in a final state
    assert_eq!(
        sm.process_event(Events::Login(&good_entry)),
        Err(Error::InvalidEvent)
    );
    assert_eq!(sm.context().attempts, 4);
    assert!(matches!(sm.state(), Ok(&States::LoginDenied)));
}
#[test]
fn guarded_transition_before_unguarded() {
    use smlang::statemachine;
    statemachine! {
        transitions: {
            *State1 + Event1 [guard] / disable = State2,
            State1 + Event1 = Fault,
            State2 + Event1 = State1,
        }
    }
    struct Context {
        pub enabled: bool,
    }
    impl StateMachineContext for Context {
        fn guard(&mut self) -> Result<bool, ()> {
            Ok(self.enabled)
        }

        fn disable(&mut self) {
            self.enabled = false;
        }
    }
    let mut sm = StateMachine::new(Context { enabled: true });
    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), Ok(&States::State2)));

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), Ok(&States::State1)));

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), Ok(&States::Fault)));
}
