//! Async guards and actions example
//!
//! An example of using async guards and actions mixed with standard ones.

#![deny(missing_docs)]

use smlang::{async_trait, statemachine};

statemachine! {
    name: AsyncSimple,
    transitions: {
        *State1 + Event1 [guard1] / async action1 = State2,
        State2 + Event2 [async guard2] / async action2 = State3,
        State3 + Event3 / action3 = State4(bool),
    }
}

/// Context with member
pub struct Context {
    lock: smol::lock::RwLock<bool>,
    done: bool,
}

#[async_trait]
impl AsyncSimpleStateMachineContext for Context {
    fn guard1(&mut self) -> Result<(), ()> {
        println!("`guard1` called from sync context");
        Ok(())
    }

    async fn action1(&mut self) -> () {
        println!("`action1` called from async context");
        let mut lock = self.lock.write().await;
        *lock = true;
    }

    async fn guard2(&mut self) -> Result<(), ()> {
        println!("`guard2` called from async context");
        let mut lock = self.lock.write().await;
        *lock = false;
        Ok(())
    }

    async fn action2(&mut self) -> () {
        println!("`action2` called from async context");
        if !*self.lock.read().await {
            self.done = true;
        }
    }

    fn action3(&mut self) -> bool {
        println!("`action3` called from sync context, done = `{}`", self.done);
        self.done
    }
}

fn main() {
    smol::block_on(async {
        let mut sm = AsyncSimpleStateMachine::new(Context {
            lock: smol::lock::RwLock::new(false),
            done: false,
        });
        assert!(matches!(sm.state(), Ok(&AsyncSimpleStates::State1)));

        let r = sm.process_event(AsyncSimpleEvents::Event1).await;
        assert!(matches!(r, Ok(&AsyncSimpleStates::State2)));

        let r = sm.process_event(AsyncSimpleEvents::Event2).await;
        assert!(matches!(r, Ok(&AsyncSimpleStates::State3)));

        let r = sm.process_event(AsyncSimpleEvents::Event3).await;
        assert!(matches!(r, Ok(&AsyncSimpleStates::State4(true))));

        // Now all events will not give any change of state
        let r = sm.process_event(AsyncSimpleEvents::Event1).await;
        assert!(matches!(r, Err(AsyncSimpleError::InvalidEvent)));
        assert!(matches!(sm.state(), Ok(&AsyncSimpleStates::State4(_))));

        let r = sm.process_event(AsyncSimpleEvents::Event2).await;
        assert!(matches!(r, Err(AsyncSimpleError::InvalidEvent)));
        assert!(matches!(sm.state(), Ok(&AsyncSimpleStates::State4(_))));
    });

    // ...
}
