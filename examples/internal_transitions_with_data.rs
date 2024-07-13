//! State data example
//!
//! An example of defining internal transitions combined with wildcard input states.

#![deny(missing_docs)]
use smlang::{assert_transition_ok, statemachine};

/// State data
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct State1Data(pub i32);

/// State data
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct State3Data(pub i32);

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
fn main() {
    {
        let mut sm = StateMachine::new(Context { count: 0 }, State1Data(1));
        assert_transition_ok!(sm, Events::Event2, States::State2, Context { count: 12 }); // action12
        assert!(sm.process_event(Events::Event1).is_err()); // InvalidEvent
        assert!(sm.process_event(Events::Event2).is_err()); // InvalidEvent
        assert!(sm.process_event(Events::Event4).is_err()); // InvalidEvent
        assert_transition_ok!(
            sm,
            Events::Event3,
            States::State3(State3Data(0)),
            Context { count: 12 + 23 }
        ); // action23
        assert_transition_ok!(
            sm,
            Events::Event3,
            States::State3(State3Data(0)),
            Context { count: 12 + 23 + 3 }
        ); // action_3
        assert_transition_ok!(
            sm,
            Events::Event3,
            States::State3(State3Data(0)),
            Context {
                count: 12 + 23 + 3 + 3
            }
        ); // action_3
        assert!(sm.process_event(Events::Event1).is_err()); // InvalidEvent
        assert!(sm.process_event(Events::Event2).is_err()); // InvalidEvent
        assert!(sm.process_event(Events::Event4).is_err()); // InvalidEvent
    }
    {
        let mut sm = StateMachine::new(Context { count: 0 }, State1Data(1));
        assert_transition_ok!(
            sm,
            Events::Event3,
            States::State3(State3Data(0)),
            Context { count: 13 }
        ); // action13
        assert!(sm.process_event(Events::Event1).is_err()); // InvalidEvent
        assert!(sm.process_event(Events::Event2).is_err()); // InvalidEvent
        assert!(sm.process_event(Events::Event4).is_err()); // InvalidEvent
    }
    {
        let mut sm = StateMachine::new(Context { count: 0 }, State1Data(1));
        assert_transition_ok!(
            sm,
            Events::Event4,
            States::State4(State3Data(0)),
            Context { count: 14 }
        ); // action14
        assert_transition_ok!(
            sm,
            Events::Event1,
            States::State4(State3Data(0)),
            Context { count: 14 + 44 }
        ); // action44
        assert_transition_ok!(
            sm,
            Events::Event3,
            States::State4(State3Data(0)),
            Context { count: 14 + 44 + 3 }
        ); // action_3
    }
}

/// Context
#[derive(Debug, PartialEq, Eq)]
pub struct Context {
    count: u32,
}
impl StateMachineContext for Context {
    fn action23(&mut self) -> Result<State3Data, ()> {
        self.count += 23;
        Ok(State3Data(300))
    }

    fn action13(&mut self, d: &State1Data) -> Result<State3Data, ()> {
        self.count += 13;
        Ok(State3Data(d.0 + 313))
    }
    fn action12(&mut self, _d: &State1Data) -> Result<(), ()> {
        self.count += 12;
        Ok(())
    }
    fn action14(&mut self, d: &State1Data) -> Result<State3Data, ()> {
        self.count += 14;
        Ok(State3Data(d.0 + 314))
    }
    fn action_3(&mut self, d: &State3Data) -> Result<State3Data, ()> {
        self.count += 3;
        Ok(*d)
    }
    fn action44(&mut self, d: &State3Data) -> Result<State3Data, ()> {
        self.count += 44;
        Ok(State3Data(d.0 + 1343))
    }
}
