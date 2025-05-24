enum State {
    #[allow(non_camel_case_types)]
    Idle_0,
    #[allow(non_camel_case_types)]
    EstablishCom_1,
    #[allow(non_camel_case_types)]
    Startup_2,
    #[allow(non_camel_case_types)]
    PreOperate_3,
    #[allow(non_camel_case_types)]
    Operate_4,
    #[allow(non_camel_case_types)]
    WURQ_5,
    ComRequestCOM3_6,
    ComRequestCOM2_7,
    ComRequestCOM1_8,
    #[allow(non_camel_case_types)]
    Retry_9,
    #[cfg(feature = "iols")]
    #[allow(non_camel_case_types)]
    WaitOnReadPulse_10,
    #[cfg(feature = "iols")]
    #[allow(non_camel_case_types)]
    WaitOnPortPowerOn_11,
}

pub trait StateActions {}

pub struct StateMachine<T: StateActions> {
    state: State,
    state_actions: T,
    retry: u8,
}

impl<T: StateActions> StateMachine<T> {
    pub fn new(state_actions: T) -> Self {
        StateMachine {
            state: State::Idle_0,
            state_actions,
            retry: 0,
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
