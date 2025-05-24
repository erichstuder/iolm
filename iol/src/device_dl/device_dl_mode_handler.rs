enum State {
    #[allow(non_camel_case_types)]
    Idle_0,
    #[allow(non_camel_case_types)]
    EstablishCom_1,
    //TODO: add more
}

pub trait StateActions {}

pub struct StateMachine<'a,T: StateActions> {
    state: State,
    state_actions: &'a T,
}

impl<'a, T: StateActions> StateMachine<'a, T> {
    pub fn new(state_actions: &'a T) -> Self {
        StateMachine {
            state: State::Idle_0,
            state_actions,
        }
    }

    pub async fn run(&mut self) {
        //info!("run state machine");
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        match self.state {
            State::Idle_0 => {
                // wait on DL_SetMode with STARTUP
                // self.state = State::EstablishCom_1;
            },
            _ => {}
        }
    }
}
