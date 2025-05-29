#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

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
    // #[cfg(feature = "iols")]
    // #[allow(non_camel_case_types)]
    // WaitOnPortPowerOn_11,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Event {
    #[allow(non_camel_case_types)]
    DL_SetMode_INACTIVE,
    #[allow(non_camel_case_types)]
    DL_SetMode_STARTUP,
    //TODO: add more
}

#[derive(Debug, Copy, Clone)]
pub enum EventError {
    InvalidState(State, Event),
}

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn wait_ms(&self, duration: u64);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_event(&self) -> Event;
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn confirm_event(&self, result: Result<(), EventError>);
}

pub struct StateMachine<T: Actions> {
    state: State,
    state_actions: T,
    //retry: u8,
}

impl<T: Actions> StateMachine<T> {
    pub fn new(state_actions: T) -> Self {
        Self {
            state: State::Idle_0,
            state_actions,
            //retry: 0,
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
                let event = self.state_actions.await_event().await;
                if event == Event::DL_SetMode_STARTUP {
                    self.state_actions.confirm_event(Ok(())).await;
                }
                else {
                    self.state_actions.confirm_event(Err(EventError::InvalidState(State::Idle_0, event))).await;
                }

                info!("enter State::EstablishCom_1");
                self.state = State::EstablishCom_1;
            },
            State::EstablishCom_1 => {
                self.state_actions.wait_ms(1000).await;
            },
        }
    }
}
