//! Async guards and actions example
//!
//! An example of using async guards and actions mixed with standard ones.

#![deny(missing_docs)]

use smlang::statemachine;

statemachine! {
    transitions: {
        *State1 + Event1 [guard1] / async action1 = State2,
        State2 + Event2 [async guard2 && guard3] / async action2 = State3,
        State3 + Event3 / action3 = State4(bool),
    }
}

/// Context with member
pub struct Context {
    lock: smol::lock::RwLock<bool>,
    done: bool,
}

impl StateMachineContext for Context {
    fn guard3(&self) -> Result<bool, ()> {
        println!("`guard3` called from async context");
        Ok(true)
    }

    async fn guard2(&self) -> Result<bool, ()> {
        println!("`guard2` called from async context");
        let mut lock = self.lock.write().await;
        *lock = false;
        Ok(true)
    }

    fn guard1(&self) -> Result<bool, ()> {
        println!("`guard1` called from sync context");
        Ok(true)
    }

    async fn action2(&mut self) -> Result<(), ()> {
        println!("`action2` called from async context");
        if !*self.lock.read().await {
            self.done = true;
        }
        Ok(())
    }

    async fn action1(&mut self) -> Result<(), ()> {
        println!("`action1` called from async context");
        let mut lock = self.lock.write().await;
        *lock = true;
        Ok(())
    }

    fn action3(&mut self) -> Result<bool, ()> {
        println!("`action3` called from sync context, done = `{}`", self.done);
        Ok(self.done)
    }
}

fn main() {
    smol::block_on(async {
        let mut sm = StateMachine::new(Context {
            lock: smol::lock::RwLock::new(false),
            done: false,
        });
        assert!(matches!(sm.state(), &States::State1));

        let r = sm.process_event(Events::Event1).await;
        assert!(matches!(r, Ok(&States::State2)));

        let r = sm.process_event(Events::Event2).await;
        assert!(matches!(r, Ok(&States::State3)));

        let r = sm.process_event(Events::Event3).await;
        assert!(matches!(r, Ok(&States::State4(true))));

        // Now all events will not give any change of state
        let r = sm.process_event(Events::Event1).await;
        assert!(matches!(r, Err(Error::InvalidEvent)));
        assert!(matches!(sm.state(), &States::State4(_)));

        let r = sm.process_event(Events::Event2).await;
        assert!(matches!(r, Err(Error::InvalidEvent)));
        assert!(matches!(sm.state(), &States::State4(_)));
    });

    // ...
}
