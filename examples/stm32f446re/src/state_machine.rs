use defmt::*;

enum State {
    State1,
    State2,
    State3,
}

pub trait StateActions {
    async fn wait_ms(&self, duration: u64);
}

pub struct StateMachine<'a, T: StateActions> {
    state: State,
    state_actions: &'a T,
}

impl<'a, T: StateActions> StateMachine<'a, T> {
    pub fn new(state_actions: &'a T) -> Self {
        StateMachine {
            state: State::State1,
            state_actions
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        match self.state {
            State::State1 => {
                info!("State 1");
                self.state_actions.wait_ms(777).await;
                self.state = State::State2;
            }
            State::State2 => {
                info!("State 2");
                self.state_actions.wait_ms(2222).await;
                self.state = State::State3;
            }
            State::State3 => {
                info!("State 3");
                self.state_actions.wait_ms(123).await;
                self.state = State::State1;
            }
        }
    }
}
