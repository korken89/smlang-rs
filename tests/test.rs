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
    /// Asserts that processing an event succeeds and settles in the expected state, which is the
    /// state after all eventless transitions. Takes the call that processes the event.
    #[macro_export]
    macro_rules! assert_settles {
        ($result:expr, $expected_state:pat) => {
            match $result {
                Ok(&$expected_state) => {}
                result => panic!(
                    "Assertion failed:\n expected: Ok(&{}),\n actual:   {:?}",
                    stringify!($expected_state),
                    result
                ),
            }
        };
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
        events_attr: #[derive(Debug,Display)],
        states_attr: #[derive(Debug,Display)],
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
        events_attr: #[derive(Debug,Display)],
        states_attr: #[derive(Debug,Display)],
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
fn async_on_entry_and_exit() {
    use smol;

    smol::block_on(async {
        statemachine! {
            entry_exit_async: true,
            transitions: {
                *State1 + Event1 = State2,
                State2 + Event2 = State3,
            }
        }

        struct Context;

        impl StateMachineContext for Context {
            async fn on_entry_state1(&mut self) {}

            async fn on_exit_state1(&mut self) {}

            async fn on_entry_state2(&mut self) {}

            async fn on_exit_state2(&mut self) {}
        }

        let mut sm = StateMachine::new(Context);

        sm.process_event(Events::Event1).await.unwrap();
        assert!(matches!(sm.state(), &States::State2));

        sm.process_event(Events::Event2).await.unwrap();
        assert!(matches!(sm.state(), &States::State3));
    });
}

#[test]
fn guard_expressions() {
    #[derive(PartialEq, Display)]
    pub struct Entry(pub u32);

    statemachine! {
        states_attr: #[derive(Display, Debug)],
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
        states_attr: #[derive(Debug, Clone,  Copy, Eq)]
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
        states_attr: #[derive(Debug, Clone,  Copy)]
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
    use serde::Serialize;
    statemachine! {
        transitions: {
            *State1 + tostate2 = State2,
            State2 + tostate3 / increment_count = State3
        },
        states_attr: #[derive(Debug, Clone, Copy, Serialize)] #[non_exhaustive] #[repr(u8)] #[serde(tag="type")],
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

/// Records what the hooks, guards and actions of a state machine do, in call order. The hooks
/// take `&self`, so the log needs interior mutability.
#[derive(Default)]
struct Log(core::cell::RefCell<Vec<String>>);

impl Log {
    fn push(&self, entry: impl Into<String>) {
        self.0.borrow_mut().push(entry.into());
    }

    fn entries(&self) -> Vec<String> {
        self.0.borrow().clone()
    }

    /// Returns the entries and clears the log, so that the next assertion only sees what was
    /// logged after this call.
    fn take(&self) -> Vec<String> {
        core::mem::take(&mut *self.0.borrow_mut())
    }

    /// Logs a transition, as seen by `transition_callback`.
    fn transition(&self, old_state: &impl core::fmt::Debug, new_state: &impl core::fmt::Debug) {
        self.push(format!("{:?} -> {:?}", old_state, new_state));
    }
}

#[test]
fn eventless_configuration_check() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + StartEvent / configure = CheckConfiguration,
            CheckConfiguration [configuration_successful] = Ready,
            CheckConfiguration = Error,
        }
    }

    struct Context {
        setting: u32,
        configuration_valid: bool,
        log: Log,
    }

    impl StateMachineContext for Context {
        fn configure(&mut self) -> Result<(), ()> {
            self.configuration_valid = self.setting != 0;
            self.log.push("configure");
            Ok(())
        }

        fn configuration_successful(&self) -> Result<bool, ()> {
            self.log.push("configuration_successful");
            Ok(self.configuration_valid)
        }

        fn on_exit_check_configuration(&mut self) {
            self.log.push("on_exit_check_configuration");
        }

        fn on_entry_ready(&mut self) {
            self.log.push("on_entry_ready");
        }

        fn transition_callback(&self, old_state: &States, new_state: &States) {
            self.log.transition(old_state, new_state);
        }
    }

    // A valid configuration ends in `Ready`, and `process_event` returns the settled state
    let mut sm = StateMachine::new(Context {
        setting: 42,
        configuration_valid: false,
        log: Log::default(),
    });
    assert_settles!(sm.process_event(Events::StartEvent), States::Ready);
    assert_eq!(
        sm.context().log.entries(),
        [
            "configure",
            "Idle -> CheckConfiguration",
            "configuration_successful",
            "on_exit_check_configuration",
            "CheckConfiguration -> Ready",
            "on_entry_ready",
        ]
    );
    // `Ready` has no event transitions
    assert_eq!(
        sm.process_event(Events::StartEvent),
        Err(Error::InvalidEvent)
    );
    assert!(matches!(sm.state(), &States::Ready));

    // An invalid configuration falls through to the unguarded transition into `Error`
    let mut sm = StateMachine::new(Context {
        setting: 0,
        configuration_valid: false,
        log: Log::default(),
    });
    assert_settles!(sm.process_event(Events::StartEvent), States::Error);
    assert_eq!(
        sm.context().log.entries(),
        [
            "configure",
            "Idle -> CheckConfiguration",
            "configuration_successful",
            "on_exit_check_configuration",
            "CheckConfiguration -> Error",
        ]
    );

    // A machine created in `CheckConfiguration` has not taken any eventless transition yet, and
    // the state has no event transitions: only `process_eventless` can settle it
    let mut sm = StateMachine::new_with_state(
        Context {
            setting: 0,
            configuration_valid: true,
            log: Log::default(),
        },
        States::CheckConfiguration,
    );
    assert_eq!(
        sm.process_event(Events::StartEvent),
        Err(Error::InvalidEvent)
    );
    assert!(sm.context().log.take().is_empty());
    assert_settles!(sm.process_eventless(), States::Ready);
    assert_eq!(
        sm.context().log.take(),
        [
            "configuration_successful",
            "on_exit_check_configuration",
            "CheckConfiguration -> Ready",
            "on_entry_ready",
        ]
    );

    let mut sm = StateMachine::new_with_state(
        Context {
            setting: 0,
            configuration_valid: false,
            log: Log::default(),
        },
        States::CheckConfiguration,
    );
    assert_settles!(sm.process_eventless(), States::Error);
    assert_eq!(
        sm.context().log.take(),
        [
            "configuration_successful",
            "on_exit_check_configuration",
            "CheckConfiguration -> Error",
        ]
    );
}

#[test]
fn eventless_chain() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Init + E / a = StepOne,
            StepOne / b = StepTwo,
            StepTwo / c = StepThree,
        }
    }

    struct Context {
        log: Log,
    }

    impl StateMachineContext for Context {
        fn a(&mut self) -> Result<(), ()> {
            self.log.push("a");
            Ok(())
        }

        fn b(&mut self) -> Result<(), ()> {
            self.log.push("b");
            Ok(())
        }

        fn c(&mut self) -> Result<(), ()> {
            self.log.push("c");
            Ok(())
        }

        fn on_exit_init(&mut self) {
            self.log.push("on_exit_init");
        }

        fn on_entry_step_one(&mut self) {
            self.log.push("on_entry_step_one");
        }

        fn on_exit_step_one(&mut self) {
            self.log.push("on_exit_step_one");
        }

        fn on_entry_step_two(&mut self) {
            self.log.push("on_entry_step_two");
        }

        fn on_exit_step_two(&mut self) {
            self.log.push("on_exit_step_two");
        }

        fn on_entry_step_three(&mut self) {
            self.log.push("on_entry_step_three");
        }

        fn transition_callback(&self, old_state: &States, new_state: &States) {
            self.log.transition(old_state, new_state);
        }
    }

    let mut sm = StateMachine::new(Context {
        log: Log::default(),
    });

    // One event runs the whole chain
    assert_settles!(sm.process_event(Events::E), States::StepThree);
    assert_eq!(
        sm.context().log.entries(),
        [
            "on_exit_init",
            "a",
            "Init -> StepOne",
            "on_entry_step_one",
            "on_exit_step_one",
            "b",
            "StepOne -> StepTwo",
            "on_entry_step_two",
            "on_exit_step_two",
            "c",
            "StepTwo -> StepThree",
            "on_entry_step_three",
        ]
    );

    // `StepThree` is final
    assert_eq!(sm.process_event(Events::E), Err(Error::InvalidEvent));
    assert!(matches!(sm.state(), &States::StepThree));
}

#[test]
fn eventless_rest_when_no_guard_passes() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start = Waiting,
            Waiting [ready] = Done,
            Waiting + Cancel = Idle,
            Done + Reset = Idle,
        }
    }

    struct Context {
        ready: bool,
        log: Log,
    }

    impl StateMachineContext for Context {
        fn ready(&self) -> Result<bool, ()> {
            self.log.push("ready");
            Ok(self.ready)
        }

        fn on_exit_waiting(&mut self) {
            self.log.push("on_exit_waiting");
        }

        fn on_entry_done(&mut self) {
            self.log.push("on_entry_done");
        }
    }

    let mut sm = StateMachine::new(Context {
        ready: false,
        log: Log::default(),
    });

    // The guard fails: the machine rests in `Waiting`, which is not an error
    assert_settles!(sm.process_event(Events::Start), States::Waiting);
    assert_eq!(sm.context().log.take(), ["ready"]);

    // `process_eventless` polls the resting state: it evaluates the guard and runs nothing else
    assert_settles!(sm.process_eventless(), States::Waiting);
    assert_settles!(sm.process_eventless(), States::Waiting);
    assert_eq!(sm.context().log.take(), ["ready", "ready"]);

    // Once the guard passes, the next poll takes the transition
    sm.context_mut().ready = true;
    assert_settles!(sm.process_eventless(), States::Done);
    assert_eq!(
        sm.context().log.take(),
        ["ready", "on_exit_waiting", "on_entry_done"]
    );
    assert_settles!(sm.process_event(Events::Reset), States::Idle);

    // The guard is not evaluated at the start of `process_event`, so the event transition
    // from `Waiting` still works when the guard would pass now
    sm.context_mut().ready = false;
    assert_settles!(sm.process_event(Events::Start), States::Waiting);
    assert_eq!(sm.context().log.take(), ["ready"]);
    sm.context_mut().ready = true;
    assert_settles!(sm.process_event(Events::Cancel), States::Idle);
    assert_eq!(sm.context().log.take(), ["on_exit_waiting"]);

    // Entering `Waiting` again evaluates the guard, which passes now
    assert_settles!(sm.process_event(Events::Start), States::Done);
    assert_eq!(
        sm.context().log.take(),
        ["ready", "on_exit_waiting", "on_entry_done"]
    );
    assert_settles!(sm.process_event(Events::Reset), States::Idle);
}

#[test]
fn eventless_polling_after_internal_transition() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start = Polling,
            Polling + Tick / work = _,
            Polling [done] = Finished,
        }
    }

    struct Context {
        count: u32,
        log: Log,
    }

    impl StateMachineContext for Context {
        fn work(&mut self) -> Result<(), ()> {
            self.count += 1;
            self.log.push("work");
            Ok(())
        }

        fn done(&self) -> Result<bool, ()> {
            self.log.push("done");
            Ok(self.count >= 3)
        }

        fn on_entry_polling(&mut self) {
            self.log.push("on_entry_polling");
        }

        fn on_exit_polling(&mut self) {
            self.log.push("on_exit_polling");
        }

        fn on_entry_finished(&mut self) {
            self.log.push("on_entry_finished");
        }
    }

    let mut sm = StateMachine::new(Context {
        count: 0,
        log: Log::default(),
    });

    // The check runs when `Polling` is entered
    assert_settles!(sm.process_event(Events::Start), States::Polling);
    assert_eq!(sm.context().log.entries(), ["on_entry_polling", "done"]);

    // It runs again after each internal transition, which does not enter or exit `Polling`
    assert_settles!(sm.process_event(Events::Tick), States::Polling);
    assert_settles!(sm.process_event(Events::Tick), States::Polling);
    assert_eq!(sm.context().count, 2);

    // The machine moves to `Finished` on the `Tick` where `done` becomes true
    assert_settles!(sm.process_event(Events::Tick), States::Finished);
    assert_eq!(sm.context().count, 3);
    assert_eq!(
        sm.context().log.entries(),
        [
            "on_entry_polling",
            "done",
            "work",
            "done",
            "work",
            "done",
            "work",
            "done",
            "on_exit_polling",
            "on_entry_finished",
        ]
    );

    // `Finished` has no event transitions
    assert_eq!(sm.process_event(Events::Tick), Err(Error::InvalidEvent));
}

#[test]
fn eventless_guarded_self_loop() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start = Working,
            Working [!done] / work = Working,
            Working = Finished,
        }
    }

    struct Context {
        count: u32,
        limit: u32,
        log: Log,
    }

    impl StateMachineContext for Context {
        fn work(&mut self) -> Result<(), ()> {
            self.count += 1;
            self.log.push("work");
            Ok(())
        }

        fn done(&self) -> Result<bool, ()> {
            Ok(self.count >= self.limit)
        }

        fn on_entry_working(&mut self) {
            self.log.push("on_entry_working");
        }

        fn on_exit_working(&mut self) {
            self.log.push("on_exit_working");
        }

        fn transition_callback(&self, old_state: &States, new_state: &States) {
            self.log.transition(old_state, new_state);
        }
    }

    let mut sm = StateMachine::new(Context {
        count: 0,
        limit: 5,
        log: Log::default(),
    });
    assert_settles!(sm.process_event(Events::Start), States::Finished);
    assert_eq!(sm.context().count, 5);

    // The self-loop neither exits nor enters `Working`, and does not call the transition callback
    let mut expected = vec!["Idle -> Working", "on_entry_working"];
    expected.extend(["work"; 5]);
    expected.extend(["on_exit_working", "Working -> Finished"]);
    assert_eq!(sm.context().log.entries(), expected);

    // The loop can end before the first iteration
    let mut sm = StateMachine::new(Context {
        count: 0,
        limit: 0,
        log: Log::default(),
    });
    assert_settles!(sm.process_event(Events::Start), States::Finished);
    assert_eq!(sm.context().count, 0);
    assert_eq!(
        sm.context().log.entries(),
        [
            "Idle -> Working",
            "on_entry_working",
            "on_exit_working",
            "Working -> Finished"
        ]
    );
}

#[test]
fn eventless_state_data() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start(u32) / begin = Measuring(u32),
            // The guard and the actions read the data of the source state, and the action
            // creates the data of the target state
            Measuring(u32) [is_large] / scale = Done(u64),
            Measuring(u32) / reject = Rejected,
            Done(u64) / archive = Archived((u64, u64)),
        }
    }

    struct Context {
        log: Log,
    }

    impl StateMachineContext for Context {
        fn begin(&mut self, event_data: u32) -> Result<u32, ()> {
            Ok(event_data)
        }

        fn is_large(&self, state_data: &u32) -> Result<bool, ()> {
            self.log.push(format!("is_large({})", state_data));
            Ok(*state_data >= 100)
        }

        fn scale(&mut self, state_data: &u32) -> Result<u64, ()> {
            self.log.push(format!("scale({})", state_data));
            Ok(u64::from(*state_data) * 1000)
        }

        fn reject(&mut self, state_data: &u32) -> Result<(), ()> {
            self.log.push(format!("reject({})", state_data));
            Ok(())
        }

        fn archive(&mut self, state_data: &u64) -> Result<(u64, u64), ()> {
            self.log.push(format!("archive({})", state_data));
            Ok((*state_data, *state_data + 1))
        }
    }

    // The guard fails, so the unguarded transition handles the state data
    let mut sm = StateMachine::new(Context {
        log: Log::default(),
    });
    assert_settles!(sm.process_event(Events::Start(5)), States::Rejected);
    assert_eq!(sm.context().log.entries(), ["is_large(5)", "reject(5)"]);

    // The state data is passed on through two eventless transitions
    let mut sm = StateMachine::new(Context {
        log: Log::default(),
    });
    assert_settles!(
        sm.process_event(Events::Start(200)),
        States::Archived((200_000, 200_001))
    );
    assert_eq!(
        sm.context().log.entries(),
        ["is_large(200)", "scale(200)", "archive(200000)"]
    );
}

#[test]
fn eventless_reference_state_data() {
    #[derive(Debug)]
    pub struct Config {
        enabled: bool,
    }

    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Load(&'a Config) / store = Loaded(&'a Config),
            Loaded(&'a Config) [is_enabled] / activate = Active(&'a Config),
            Loaded(&'a Config) = Disabled,
        }
    }

    struct Context;

    impl StateMachineContext for Context {
        fn store<'a>(&mut self, event_data: &'a Config) -> Result<&'a Config, ()> {
            Ok(event_data)
        }

        fn is_enabled(&self, state_data: &Config) -> Result<bool, ()> {
            Ok(state_data.enabled)
        }

        fn activate<'a>(&mut self, state_data: &'a Config) -> Result<&'a Config, ()> {
            Ok(state_data)
        }
    }

    let enabled = Config { enabled: true };
    let mut sm = StateMachine::new(Context);
    match sm.process_event(Events::Load(&enabled)) {
        Ok(States::Active(config)) => assert!(core::ptr::eq(*config, &enabled)),
        other => panic!("unexpected result: {:?}", other),
    }

    let disabled = Config { enabled: false };
    let mut sm = StateMachine::new(Context);
    assert_settles!(sm.process_event(Events::Load(&disabled)), States::Disabled);
}

#[test]
fn eventless_temporary_context() {
    statemachine! {
        temporary_context: &mut u32,
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start = Check,
            Check [ready] / finish = Done,
            Check = Fail,
        }
    }

    struct Context;

    impl StateMachineContext for Context {
        fn ready(&self, temporary_context: &mut u32) -> Result<bool, ()> {
            *temporary_context += 1;
            Ok(*temporary_context >= 2)
        }

        fn finish(&mut self, temporary_context: &mut u32) -> Result<(), ()> {
            *temporary_context += 10;
            Ok(())
        }
    }

    // The guard fails and the fallback has no action
    let mut counter = 0;
    let mut sm = StateMachine::new(Context);
    assert_settles!(sm.process_event(&mut counter, Events::Start), States::Fail);
    assert_eq!(counter, 1);

    // The guard passes, and the action gets the temporary context as well
    let mut counter = 1;
    let mut sm = StateMachine::new(Context);
    assert_settles!(sm.process_event(&mut counter, Events::Start), States::Done);
    assert_eq!(counter, 12);

    // `process_eventless` takes the temporary context, as the guard and the action need it
    let mut counter = 1;
    let mut sm = StateMachine::new_with_state(Context, States::Check);
    assert_settles!(sm.process_eventless(&mut counter), States::Done);
    assert_eq!(counter, 12);
}

#[test]
fn eventless_async_guard_and_action() {
    smol::block_on(async {
        statemachine! {
            states_attr: #[derive(Debug)],
            transitions: {
                *Idle + Start = Check,
                Check [async ready] / async finish = Done,
                Check = Fail,
            }
        }

        struct Context {
            ready: bool,
            log: Log,
        }

        impl StateMachineContext for Context {
            async fn ready(&self) -> Result<bool, ()> {
                self.log.push("ready");
                Ok(self.ready)
            }

            async fn finish(&mut self) -> Result<(), ()> {
                self.log.push("finish");
                Ok(())
            }
        }

        // Only the eventless transitions are async, but `process_event` is async as well
        let mut sm = StateMachine::new(Context {
            ready: true,
            log: Log::default(),
        });
        assert_settles!(sm.process_event(Events::Start).await, States::Done);
        assert_eq!(sm.context().log.entries(), ["ready", "finish"]);

        let mut sm = StateMachine::new(Context {
            ready: false,
            log: Log::default(),
        });
        assert_settles!(sm.process_event(Events::Start).await, States::Fail);
        assert_eq!(sm.context().log.entries(), ["ready"]);

        // `process_eventless` is async as well
        let mut sm = StateMachine::new_with_state(
            Context {
                ready: true,
                log: Log::default(),
            },
            States::Check,
        );
        assert_settles!(sm.process_eventless().await, States::Done);
        assert_eq!(sm.context().log.take(), ["ready", "finish"]);
        assert_settles!(sm.process_eventless().await, States::Done);
        assert!(sm.context().log.take().is_empty());
    });
}

#[test]
fn eventless_async_entry_and_exit() {
    smol::block_on(async {
        statemachine! {
            entry_exit_async: true,
            states_attr: #[derive(Debug)],
            transitions: {
                *Idle + Start = Check,
                Check [ready] / finish = Done,
                Check = Fail,
            }
        }

        struct Context {
            log: Log,
        }

        impl StateMachineContext for Context {
            fn ready(&self) -> Result<bool, ()> {
                self.log.push("ready");
                Ok(true)
            }

            fn finish(&mut self) -> Result<(), ()> {
                self.log.push("finish");
                Ok(())
            }

            async fn on_exit_idle(&mut self) {
                self.log.push("on_exit_idle");
            }

            async fn on_entry_check(&mut self) {
                self.log.push("on_entry_check");
            }

            async fn on_exit_check(&mut self) {
                self.log.push("on_exit_check");
            }

            async fn on_entry_done(&mut self) {
                self.log.push("on_entry_done");
            }
        }

        let mut sm = StateMachine::new(Context {
            log: Log::default(),
        });
        assert_settles!(sm.process_event(Events::Start).await, States::Done);
        assert_eq!(
            sm.context().log.take(),
            [
                "on_exit_idle",
                "on_entry_check",
                "ready",
                "on_exit_check",
                "finish",
                "on_entry_done",
            ]
        );

        // `Done` has no eventless transitions, so `process_eventless` does nothing there
        assert_settles!(sm.process_eventless().await, States::Done);
        assert!(sm.context().log.take().is_empty());
    });
}

#[test]
fn eventless_errors_are_not_rolled_back() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Init + E / a = Prepare,
            Prepare / b = Verify,
            Verify [check] = Done,
            Verify = Fail,
        }
    }

    struct Context {
        fail_b: bool,
        fail_check: bool,
        log: Log,
    }

    impl StateMachineContext for Context {
        fn a(&mut self) -> Result<(), ()> {
            self.log.push("a");
            Ok(())
        }

        fn b(&mut self) -> Result<(), ()> {
            self.log.push("b");
            if self.fail_b {
                Err(())
            } else {
                Ok(())
            }
        }

        fn check(&self) -> Result<bool, ()> {
            self.log.push("check");
            if self.fail_check {
                Err(())
            } else {
                Ok(true)
            }
        }

        fn on_exit_prepare(&mut self) {
            self.log.push("on_exit_prepare");
        }

        fn on_entry_verify(&mut self) {
            self.log.push("on_entry_verify");
        }

        fn transition_callback(&self, old_state: &States, new_state: &States) {
            self.log.transition(old_state, new_state);
        }
    }

    // An action fails: the event transition stays committed, and the machine stays in the
    // source state of the failed eventless transition. Like for event transitions, its
    // `on_exit` hook has already run.
    let mut sm = StateMachine::new(Context {
        fail_b: true,
        fail_check: false,
        log: Log::default(),
    });
    assert_eq!(sm.process_event(Events::E), Err(Error::ActionFailed(())));
    assert!(matches!(sm.state(), &States::Prepare));
    assert_eq!(
        sm.context().log.take(),
        ["a", "Init -> Prepare", "on_exit_prepare", "b"]
    );

    // Sending `E` again does not retry: `Prepare` has no event transitions. Only
    // `process_eventless` takes the failed eventless transition again.
    assert_eq!(sm.process_event(Events::E), Err(Error::InvalidEvent));
    assert!(matches!(sm.state(), &States::Prepare));

    // The retry runs `on_exit_prepare` and the action again, and then the rest of the chain
    sm.context_mut().fail_b = false;
    assert_settles!(sm.process_eventless(), States::Done);
    assert_eq!(
        sm.context().log.take(),
        [
            "on_exit_prepare",
            "b",
            "Prepare -> Verify",
            "on_entry_verify",
            "check",
            "Verify -> Done",
        ]
    );

    // A guard fails after the first eventless transition: it stays committed as well
    let mut sm = StateMachine::new(Context {
        fail_b: false,
        fail_check: true,
        log: Log::default(),
    });
    assert_eq!(sm.process_event(Events::E), Err(Error::GuardFailed(())));
    assert!(matches!(sm.state(), &States::Verify));
    assert_eq!(
        sm.context().log.take(),
        [
            "a",
            "Init -> Prepare",
            "on_exit_prepare",
            "b",
            "Prepare -> Verify",
            "on_entry_verify",
            "check",
        ]
    );

    // The same retry works after a failed guard, and starts at the guard
    sm.context_mut().fail_check = false;
    assert_settles!(sm.process_eventless(), States::Done);
    assert_eq!(sm.context().log.take(), ["check", "Verify -> Done"]);
}

#[test]
fn eventless_process_eventless_with_an_owned_temporary_context() {
    statemachine! {
        temporary_context: String,
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start / configure = Flush,
            Flush = Done,
        }
    }

    struct Context {
        log: Log,
    }

    impl StateMachineContext for Context {
        // The action consumes the temporary context. The eventless transition has no guard
        // and no action, so `process_event` does not pass the context on to it.
        fn configure(&mut self, temporary_context: String) -> Result<(), ()> {
            self.log.push(temporary_context);
            Ok(())
        }
    }

    let mut sm = StateMachine::new(Context {
        log: Log::default(),
    });
    assert_settles!(
        sm.process_event(String::from("configured"), Events::Start),
        States::Done
    );
    assert_eq!(sm.context().log.entries(), ["configured"]);

    // `process_eventless` takes no temporary context, as none of its guards and actions does
    assert_settles!(sm.process_eventless(), States::Done);
}

#[test]
fn eventless_custom_error() {
    statemachine! {
        custom_error: true,
        states_attr: #[derive(Debug)],
        transitions: {
            *A + E = B,
            B [check] / finish = C,
        }
    }

    #[derive(Debug, PartialEq)]
    enum MyError {
        Check,
        Finish,
    }

    struct Context {
        fail_check: bool,
        fail_finish: bool,
    }

    impl StateMachineContext for Context {
        type Error = MyError;

        fn check(&self) -> Result<bool, Self::Error> {
            if self.fail_check {
                Err(MyError::Check)
            } else {
                Ok(true)
            }
        }

        fn finish(&mut self) -> Result<(), Self::Error> {
            if self.fail_finish {
                Err(MyError::Finish)
            } else {
                Ok(())
            }
        }
    }

    let mut sm = StateMachine::new(Context {
        fail_check: true,
        fail_finish: false,
    });
    assert_eq!(
        sm.process_event(Events::E),
        Err(Error::GuardFailed(MyError::Check))
    );
    assert!(matches!(sm.state(), &States::B));

    let mut sm = StateMachine::new(Context {
        fail_check: false,
        fail_finish: true,
    });
    assert_eq!(
        sm.process_event(Events::E),
        Err(Error::ActionFailed(MyError::Finish))
    );
    assert!(matches!(sm.state(), &States::B));

    let mut sm = StateMachine::new(Context {
        fail_check: false,
        fail_finish: false,
    });
    assert_settles!(sm.process_event(Events::E), States::C);
}

#[test]
fn eventless_named_state_machine() {
    statemachine! {
        name: Foo,
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start / configure = Check,
            Check [valid] = Ready,
            Check = Failed,
        }
    }

    struct Context {
        valid: bool,
    }

    impl FooStateMachineContext for Context {
        fn configure(&mut self) -> Result<(), ()> {
            Ok(())
        }

        fn valid(&self) -> Result<bool, ()> {
            Ok(self.valid)
        }
    }

    let mut sm = FooStateMachine::new(Context { valid: true });
    assert_settles!(sm.process_event(FooEvents::Start), FooStates::Ready);
    assert_eq!(
        sm.process_event(FooEvents::Start),
        Err(FooError::InvalidEvent)
    );

    let mut sm = FooStateMachine::new(Context { valid: false });
    assert_settles!(sm.process_event(FooEvents::Start), FooStates::Failed);
}

#[test]
fn eventless_with_wildcards() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start = Check,
            Check [ok] = Ready,
            Check = Error,
            _ + Reset = Idle,
            _ + Poke / poke = _,
        }
    }

    struct Context {
        ok: bool,
        pokes: u32,
    }

    impl StateMachineContext for Context {
        fn ok(&self) -> Result<bool, ()> {
            Ok(self.ok)
        }

        fn poke(&mut self) -> Result<(), ()> {
            self.pokes += 1;
            Ok(())
        }
    }

    let mut sm = StateMachine::new(Context { ok: true, pokes: 0 });

    // `Reset` works in every state, also in the states that are only used by eventless
    // transitions
    assert_settles!(sm.process_event(Events::Start), States::Ready);
    assert_settles!(sm.process_event(Events::Reset), States::Idle);
    sm.context_mut().ok = false;
    assert_settles!(sm.process_event(Events::Start), States::Error);
    assert_settles!(sm.process_event(Events::Reset), States::Idle);

    // A machine can be placed in `Check` by `new_with_state`. Eventless transitions are not
    // evaluated for it, and `Check` has no event transitions of its own
    let mut sm = StateMachine::new_with_state(Context { ok: true, pokes: 0 }, States::Check);
    assert_eq!(sm.process_event(Events::Start), Err(Error::InvalidEvent));
    assert!(matches!(sm.state(), &States::Check));

    // An internal transition triggers the eventless transitions of its state
    assert_settles!(sm.process_event(Events::Poke), States::Ready);
    assert_eq!(sm.context().pokes, 1);
    assert_settles!(sm.process_event(Events::Poke), States::Ready);
    assert_eq!(sm.context().pokes, 2);

    let mut sm = StateMachine::new_with_state(Context { ok: true, pokes: 0 }, States::Check);
    assert_settles!(sm.process_event(Events::Reset), States::Idle);
}

#[test]
fn eventless_logging_hooks() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start = Check,
            Check [ok] / act = Done,
            Check = Fail,
            Done / finish = End,
        }
    }

    struct Context {
        ok: bool,
        log: Log,
    }

    impl StateMachineContext for Context {
        fn ok(&self) -> Result<bool, ()> {
            Ok(self.ok)
        }

        fn act(&mut self) -> Result<(), ()> {
            Ok(())
        }

        fn finish(&mut self) -> Result<(), ()> {
            Ok(())
        }

        fn log_process_event(&self, _current_state: &States, _event: &Events) {
            self.log.push("process_event");
        }

        fn log_guard(&self, _guard: &'static str, result: bool) {
            self.log.push(format!("guard {}", result));
        }

        fn log_action(&self, action: &'static str) {
            self.log.push(format!("action {}", action));
        }
    }

    // `Idle` has no eventless transitions, so `process_eventless` does nothing there and logs
    // nothing. The event transitions are not affected.
    let mut sm = StateMachine::new(Context {
        ok: true,
        log: Log::default(),
    });
    assert_settles!(sm.process_eventless(), States::Idle);
    assert!(sm.context().log.take().is_empty());

    // Guards and actions of eventless transitions are logged, but there is only one
    // `process_event` entry for the whole chain
    assert_settles!(sm.process_event(Events::Start), States::End);
    assert_eq!(
        sm.context().log.take(),
        ["process_event", "guard true", "action act", "action finish"]
    );

    // `End` has no eventless transitions either
    assert_settles!(sm.process_eventless(), States::End);
    assert!(sm.context().log.take().is_empty());

    let mut sm = StateMachine::new(Context {
        ok: false,
        log: Log::default(),
    });
    assert_settles!(sm.process_event(Events::Start), States::Fail);
    assert_eq!(sm.context().log.take(), ["process_event", "guard false"]);

    // `process_eventless` logs the guards and actions it runs, but no `process_event` entry
    let mut sm = StateMachine::new_with_state(
        Context {
            ok: true,
            log: Log::default(),
        },
        States::Check,
    );
    assert_settles!(sm.process_eventless(), States::End);
    assert_eq!(
        sm.context().log.take(),
        ["guard true", "action act", "action finish"]
    );

    let mut sm = StateMachine::new_with_state(
        Context {
            ok: false,
            log: Log::default(),
        },
        States::Check,
    );
    assert_settles!(sm.process_eventless(), States::Fail);
    assert_eq!(sm.context().log.take(), ["guard false"]);
}

#[test]
fn eventless_self_loop_with_state_data() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start(u32) / begin = Counting(u32),
            // The self-loop replaces the state data in each iteration
            Counting(u32) [!reached] / increment = Counting(u32),
            Counting(u32) / finish = Done(u32),
        }
    }

    struct Context {
        limit: u32,
    }

    impl StateMachineContext for Context {
        fn begin(&mut self, event_data: u32) -> Result<u32, ()> {
            Ok(event_data)
        }

        fn reached(&self, state_data: &u32) -> Result<bool, ()> {
            Ok(*state_data >= self.limit)
        }

        fn increment(&mut self, state_data: &u32) -> Result<u32, ()> {
            Ok(*state_data + 1)
        }

        fn finish(&mut self, state_data: &u32) -> Result<u32, ()> {
            Ok(*state_data * 100)
        }
    }

    let mut sm = StateMachine::new(Context { limit: 7 });
    assert_settles!(sm.process_event(Events::Start(3)), States::Done(700));
}

#[test]
fn eventless_shares_guards_and_actions_with_event_transitions() {
    statemachine! {
        states_attr: #[derive(Debug)],
        transitions: {
            *Idle + Start [ok] / act = Check,
            Check [ok] / act = Done,
            Check = Fail,
        }
    }

    struct Context {
        ok: bool,
        acts: u32,
    }

    impl StateMachineContext for Context {
        fn ok(&self) -> Result<bool, ()> {
            Ok(self.ok)
        }

        fn act(&mut self) -> Result<(), ()> {
            self.acts += 1;
            Ok(())
        }
    }

    let mut sm = StateMachine::new(Context { ok: true, acts: 0 });
    assert_settles!(sm.process_event(Events::Start), States::Done);
    assert_eq!(sm.context().acts, 2);

    let mut sm = StateMachine::new(Context { ok: false, acts: 0 });
    assert_eq!(
        sm.process_event(Events::Start),
        Err(Error::TransitionsFailed)
    );
    assert!(matches!(sm.state(), &States::Idle));
    assert_eq!(sm.context().acts, 0);
}

#[test]
fn machine_without_eventless_transitions_has_no_process_eventless() {
    statemachine! {
        transitions: {
            *Idle + Start = Done,
        }
    }

    struct Context;

    impl StateMachineContext for Context {}

    // The method only exists with eventless transitions, so the name is free otherwise. A
    // generated method of the same name would be a duplicate definition.
    impl<T: StateMachineContext> StateMachine<T> {
        fn process_eventless(&self) -> u32 {
            42
        }
    }

    let mut sm = StateMachine::new(Context);
    assert_eq!(sm.process_eventless(), 42);
    assert!(sm.process_event(Events::Start).is_ok());
}
