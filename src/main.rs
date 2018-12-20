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

// Transition DSL: src_state + event [ guard ] / action = dst_state
statemachine!(
    *State1 + Event1[guard1] / action1 = State2,
    State2 + Event2[guard2] / action2 = State3,
    State2 + Event3 = State1,
    State3 + Event3 = State1,
);

struct StateMachine {
    state: States,
}

impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            state: States::State1,
        }
    }

    pub fn state(&self) -> States {
        self.state
    }

    pub fn run(&mut self, event: Events) {
        match self.state {
            States::State1 => match event {
                Events::Event1 => {
                    println!("State1, Event1"); // Do something real in the future
                    if guard1() {
                        action1();
                        self.state = States::State2;
                    }
                }
                _ => println!("State1, {:?}, nothing happens", event),
            },
            States::State2 => match event {
                Events::Event2 => {
                    println!("State2, Event2"); // Do something real in the future
                    if guard2() {
                        action2();
                        self.state = States::State3;
                    }
                }
                Events::Event3 => {
                    println!("State2, Event1"); // Do something real in the future
                    self.state = States::State1;
                }
                _ => println!("State2, {:?}, nothing happens", event),
            },
            States::State3 => match event {
                Events::Event3 => {
                    println!("State3, Event3"); // Do something real in the future
                    self.state = States::State1;
                }
                _ => println!("State3, {:?}, nothing happens", event),
            },
        }
    }
}

fn main() {
    let mut sm = StateMachine::new();
    assert_eq!(sm.state(), States::State1);

    sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State3);

    sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State3);

    sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);

    sm.run(Events::Event3);
    assert_eq!(sm.state(), States::State1);

    sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State1);

    sm.run(Events::Event1);
    assert_eq!(sm.state(), States::State2);

    sm.run(Events::Event2);
    assert_eq!(sm.state(), States::State3);
}
