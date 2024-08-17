extern crate smlang;

use derive_more::Display;

use smlang::statemachine;

mod internal_macros {
    #[macro_export]
    macro_rules! assert_transition {
        ($sm:expr, $event:expr, $expected_state:expr, $expected_count:expr) => {{
            let prev_state = $sm.state;
            $sm.process_event($event).unwrap();
            println!("{:?} -> {:?} : {:?}", prev_state, $sm.state, $sm.context());
            assert_eq!($expected_state, $sm.state);
            assert_eq!($expected_count, $sm.context().count);
        }};
    }
    #[macro_export]
    macro_rules! assert_transition_ok {
        ($sm:expr, $event:expr, $expected_action:expr, $expected_result:pat) => {{
            let prev_state = $sm.state;
            if let Ok(result) = $sm.process_event($event) {
                let result = result.clone();
                println!("{:?} -> {:?} : {:?}", prev_state, result, $sm.context());
                match result {
                    $expected_result => {}
                    _ => panic!(
                        "Assertion failed:\n expected: {:?},\n actual:    {:?}",
                        stringify!($expected_result),
                        result
                    ),
                }
                assert_eq!($expected_action, $sm.context().action);
            } else {
                panic!("assert_transition_ok failed")
            }
        }};
    }
}

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
    assert!(matches!(sm.state(), &States::State2));

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), &States::Fault));
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
        fn guard1(&self, _event_data: &X) -> Result<bool, ()> {
            Ok(true)
        }

        fn guard2(&self, _state_data: &X, _event_data: &Y) -> Result<bool, ()> {
            Ok(true)
        }

        fn guard3(&self, _event_data: &Z) -> Result<bool, ()> {
            Ok(true)
        }

        fn action1<'a>(&mut self, event_data: &'a X) -> Result<&'a X, ()> {
            Ok(event_data)
        }

        fn action2<'a, 'b>(
            &mut self,
            state_data: &'a X,
            event_data: &'b Y,
        ) -> Result<(&'a X, &'b Y), ()> {
            Ok((state_data, event_data))
        }

        fn action3(&mut self, _event_data: &Z) -> Result<(), ()> {
            Ok(())
        }
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
    assert!(matches!(sm.state(), &States::Init));

    let event = Events::Event;
    assert_eq!(format!("{}", event), "Event");

    sm.process_event(event).unwrap();
    assert!(matches!(sm.state(), &States::End));
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
    assert!(matches!(sm.state(), &SMStates::Init));

    let event = SMEvents::Event;
    assert_eq!(format!("{}", event), "Event");

    sm.process_event(event).unwrap();
    assert!(matches!(sm.state(), &SMStates::End));
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

        impl StateMachineContext for Context {
            async fn guard1(&self) -> Result<bool, ()> {
                Ok(true)
            }

            async fn action1(&mut self) -> Result<(), ()> {
                Ok(())
            }
        }

        let mut sm = StateMachine::new(Context);

        sm.process_event(Events::Event1).await.unwrap();
        assert!(matches!(sm.state(), &States::State2));

        sm.process_event(Events::Event1).await.unwrap();
        assert!(matches!(sm.state(), &States::Fault));
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
        fn valid_entry(&self, e: &Entry) -> Result<bool, ()> {
            Ok(e.0 == self.password)
        }
        fn too_many_attempts(&self, _e: &Entry) -> Result<bool, ()> {
            Ok(self.attempts >= 3)
        }
        fn reset(&mut self) -> Result<(), ()> {
            self.attempts = 0;
            Ok(())
        }
        fn attempt(&mut self, _e: &Entry) -> Result<(), ()> {
            self.attempts += 1;
            Ok(())
        }
    }

    let mut sm = StateMachine::new(Context {
        password: 42,
        attempts: 0,
    });
    assert!(matches!(sm.state(), &States::Init));

    let bad_entry = Entry(10);
    let good_entry = Entry(42);

    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 1);
    assert!(matches!(sm.state(), &States::Init));

    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 2);
    assert!(matches!(sm.state(), &States::Init));

    let _ = sm.process_event(Events::Login(&good_entry));
    assert_eq!(sm.context().attempts, 3);
    assert!(matches!(sm.state(), &States::LoggedIn));

    let _ = sm.process_event(Events::Logout);
    assert_eq!(sm.context().attempts, 0);
    assert!(matches!(sm.state(), &States::Init));

    let _ = sm.process_event(Events::Login(&bad_entry));
    let _ = sm.process_event(Events::Login(&bad_entry));
    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 3);
    assert!(matches!(sm.state(), &States::Init));

    // exhausted attempts
    let _ = sm.process_event(Events::Login(&bad_entry));
    assert_eq!(sm.context().attempts, 4);
    assert!(matches!(sm.state(), &States::LoginDenied));

    // Invalid event, as we are in a final state
    assert_eq!(
        sm.process_event(Events::Login(&good_entry)),
        Err(Error::InvalidEvent)
    );
    assert_eq!(sm.context().attempts, 4);
    assert!(matches!(sm.state(), &States::LoginDenied));
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
        fn guard(&self) -> Result<bool, ()> {
            Ok(self.enabled)
        }

        fn disable(&mut self) -> Result<(), ()> {
            self.enabled = false;
            Ok(())
        }
    }
    let mut sm = StateMachine::new(Context { enabled: true });
    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), &States::State2));

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), &States::State1));

    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), &States::Fault));
}

#[test]
fn guard_errors() {
    use smlang::statemachine;
    statemachine! {
        transitions: {
            *Init + Event1 [guard] = Done,
        }
    }

    struct Context {
        pub guard_passable: bool,
        pub guard_errors: bool,
    }
    impl StateMachineContext for Context {
        fn guard(&self) -> Result<bool, ()> {
            if self.guard_errors {
                Err(())
            } else {
                Ok(self.guard_passable)
            }
        }
    }

    let mut sm = StateMachine::new(Context {
        guard_passable: false,
        guard_errors: true,
    });

    // Test attempting to transition when the guard fails.
    sm.context_mut().guard_errors = true;
    assert!(sm.process_event(Events::Event1).is_err());
    assert!(matches!(sm.state(), &States::Init));

    // Test attempting to transition when the guard is not passable.
    sm.context_mut().guard_errors = false;
    assert!(sm.process_event(Events::Event1).is_err());
    assert!(matches!(sm.state(), &States::Init));

    assert!(sm.process_event(Events::Event1).is_err());
    assert!(matches!(sm.state(), &States::Init));

    sm.context_mut().guard_passable = true;
    sm.process_event(Events::Event1).unwrap();
    assert!(matches!(sm.state(), &States::Done));
}
#[test]
fn test_internal_transition_with_data() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct State1Data(pub ActionId);
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct State3Data(pub ActionId);

    statemachine! {
        transitions: {
            *State1(State1Data) + Event2 / action12 = State2,
            State1(State1Data) + Event3 / action13 = State3(State3Data),
            State1(State1Data) + Event4 / action14 = State4(State3Data),

            State2 + Event3 / action23 = State3(State3Data),
            State4(State3Data) + Event1 / action44 = _, // Same as State4(State3Data) + Event1 / action44

            // TRANSITION : _ + Event3 / increment_count = _, IS EQUIVALENT TO THE FOLLOWING TWO:
            // State3(State3Data) + Event3 / action_3 = State3(State3Data),
            // State4(State3Data) + Event3 / action_3 = State4(State3Data),
            _ + Event3 / action_3 = _,
        },
        derive_states: [Debug, Clone,  Copy, Eq ]
    }
    /// Context
    #[derive(Default, Debug, PartialEq, Eq)]
    pub struct Context {
        action: ActionId,
    }
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    enum ActionId {
        #[default]
        Init,
        Action3,
        Action12,
        Action13,
        Action14,
        Action23,
        Action44,
    }
    impl StateMachineContext for Context {
        fn action_3(&mut self, d: &State3Data) -> Result<State3Data, ()> {
            self.action = ActionId::Action3;
            Ok(*d)
        }

        fn action44(&mut self, _d: &State3Data) -> Result<State3Data, ()> {
            self.action = ActionId::Action44;
            Ok(State3Data(ActionId::Action44))
        }
        fn action14(&mut self, _d: &State1Data) -> Result<State3Data, ()> {
            self.action = ActionId::Action14;
            Ok(State3Data(ActionId::Action14))
        }
        fn action12(&mut self, _d: &State1Data) -> Result<(), ()> {
            self.action = ActionId::Action12;
            Ok(())
        }
        fn action13(&mut self, _d: &State1Data) -> Result<State3Data, ()> {
            self.action = ActionId::Action13;
            Ok(State3Data(ActionId::Action13))
        }
        fn action23(&mut self) -> Result<State3Data, ()> {
            self.action = ActionId::Action23;
            Ok(State3Data(ActionId::Action23))
        }
    }

    {
        let mut sm = StateMachine::new(Context::default(), State1Data(ActionId::Init));
        matches!(States::State2, States::State2);
        assert_transition_ok!(sm, Events::Event2, ActionId::Action12, States::State2);
        assert!(sm.process_event(Events::Event1).is_err());
        assert!(sm.process_event(Events::Event2).is_err());
        assert!(sm.process_event(Events::Event4).is_err());
        assert_transition_ok!(sm, Events::Event3, ActionId::Action23, States::State3(_));
        assert_transition_ok!(sm, Events::Event3, ActionId::Action3, States::State3(_));
        assert_transition_ok!(sm, Events::Event3, ActionId::Action3, States::State3(_));
        assert!(sm.process_event(Events::Event1).is_err());
        assert!(sm.process_event(Events::Event2).is_err());
        assert!(sm.process_event(Events::Event4).is_err());
    }
    {
        let mut sm = StateMachine::new(Context::default(), State1Data(ActionId::Init));
        assert_transition_ok!(sm, Events::Event3, ActionId::Action13, States::State3(_));
        assert!(sm.process_event(Events::Event1).is_err());
        assert!(sm.process_event(Events::Event2).is_err());
        assert!(sm.process_event(Events::Event4).is_err());
    }
    {
        let mut sm = StateMachine::new(Context::default(), State1Data(ActionId::Init));
        assert_transition_ok!(sm, Events::Event4, ActionId::Action14, States::State4(_));
        assert_transition_ok!(sm, Events::Event1, ActionId::Action44, States::State4(_));
        assert_transition_ok!(sm, Events::Event3, ActionId::Action3, States::State4(_));
    }
}
#[test]
fn test_wildcard_states_and_internal_transitions() {
    statemachine! {
        transitions: {
            *State1 + Event2 = State2,
            State2 + Event3 = State3,
            _ + Event1 / increment_count,      // Internal transition (implicit: omitting target state)
            _ + Event3 / increment_count = _ , // Internal transition (explicit: using _ as target state)
        },
        derive_states: [Debug, Clone,  Copy]
    }
    #[derive(Debug)]
    pub struct Context {
        count: u32,
    }
    impl StateMachineContext for Context {
        fn increment_count(&mut self) -> Result<(), ()> {
            self.count += 1;
            Ok(())
        }
    }

    let mut sm = StateMachine::new(Context { count: 0 });

    assert_transition!(sm, Events::Event1, States::State1, 1);
    assert_transition!(sm, Events::Event2, States::State2, 1);
    assert_transition!(sm, Events::Event3, States::State3, 1);
    assert_transition!(sm, Events::Event1, States::State3, 2);
    assert_transition!(sm, Events::Event3, States::State3, 3);

    assert!(sm.process_event(Events::Event2).is_err()); // InvalidEvent
    assert_eq!(States::State3, sm.state);
}
#[test]
fn test_specify_attrs() {
    #![deny(non_camel_case_types)]
    statemachine! {
        transitions: {
            *State1 + tostate2 = State2,
            State2 + tostate3 / increment_count = State3
        },
        derive_states: [Debug, Clone, Copy],
        states_attr: #[non_exhaustive] #[repr(u8)],
        events_attr: #[derive(Debug)] #[allow(non_camel_case_types)]
    }

    #[derive(Debug, Default)]
    pub struct Context {
        count: u32,
    }

    impl StateMachineContext for Context {
        fn increment_count(&mut self) -> Result<(), ()> {
            self.count += 1;
            Ok(())
        }
    }
    let mut sm = StateMachine::new(Context::default());

    assert_eq!(sm.state().clone(), States::State1);
    assert_transition!(sm, Events::tostate2, States::State2, 0);
    assert_transition!(sm, Events::tostate3, States::State3, 1);
}
