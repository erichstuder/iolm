#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    #[allow(non_camel_case_types)]
    Idle_0,
    #[allow(non_camel_case_types)]
    EstablishCom_1,
    // #[allow(non_camel_case_types)]
    // Startup_2,
    // #[allow(non_camel_case_types)]
    // PreOperate_3,
    //#[allow(non_camel_case_types)]
    //Operate_4,
    // #[allow(non_camel_case_types)]
    // WURQ_5,
    // ComRequestCOM3_6,
    // ComRequestCOM2_7,
    // ComRequestCOM1_8,
    // #[allow(non_camel_case_types)]
    // Retry_9,
    // #[cfg(feature = "iols")]
    // #[allow(non_camel_case_types)]
    // WaitOnReadPulse_10,
    #[cfg(feature = "iols")]
    #[allow(non_camel_case_types)]
    WaitOnPortPowerOn_11,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Event {
    #[allow(non_camel_case_types)]
    DL_SetMode_INACTIVE,
    #[allow(non_camel_case_types)]
    DL_SetMode_STARTUP,
    #[allow(non_camel_case_types)]
    DL_SetMODE_PREOPERATE,
    #[allow(non_camel_case_types)]
    DL_SetMODE_OPERATE,
}

#[derive(Debug, Copy, Clone)]
pub enum EventError {
    InvalidState(State, Event),
}

enum Safety {
    #[allow(dead_code)] //TODO: remove as soon as NonSafety is assigned
    NonSafety,
    SafetyCom,
}

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn wait_ms(&self, duration: u64);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_event(&self) -> Event;
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn confirm_event(&self, result: Result<(), EventError>);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_off_on_ms(&self, duration: u64);
}

pub struct StateMachine<T: Actions> {
    state: State,
    actions: T,
    retry: u8,
    safety: Safety,
    min_shutdown_time_ms: u64,
}

impl<T: Actions> StateMachine<T> {
    pub fn new(actions: T) -> Self {
        Self {
            state: State::Idle_0,
            actions,
            retry: 0,
            safety: Safety::SafetyCom, //TODO: don't know yet where it will be set from.
            min_shutdown_time_ms: 3000, //TODO: don't know yet where it will be set from.
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        match self.state {
            State::Idle_0 => {
                let event = self.actions.await_event().await;
                match event {
                    Event::DL_SetMode_STARTUP => self.actions.confirm_event(Ok(())).await,
                    _ => self.actions.confirm_event(Err(EventError::InvalidState(State::Idle_0, event))).await,
                }
                match self.safety {
                    Safety::NonSafety => {
                        self.retry = 0;
                        self.state = State::EstablishCom_1;
                    }
                    #[cfg(feature = "iols")]
                    Safety::SafetyCom => {
                        self.actions.port_power_off_on_ms(self.min_shutdown_time_ms).await;
                        self.state = State::WaitOnPortPowerOn_11;
                    }
                }
            },
            State::EstablishCom_1 => {
                // TODO: implement
                self.actions.wait_ms(1000).await;
            },
            State::WaitOnPortPowerOn_11 => {
                // TODO: implement
                self.actions.wait_ms(1000).await;
            }

        }
    }
}
