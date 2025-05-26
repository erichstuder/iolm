use defmt::info;

#[derive(PartialEq, Copy, Clone)]
pub enum State {
    #[allow(non_camel_case_types)]
    Idle_0,
    #[allow(non_camel_case_types)]
    EstablishCom_1,
    // #[allow(non_camel_case_types)]
    // Startup_2,
    // #[allow(non_camel_case_types)]
    // PreOperate_3,
    #[allow(non_camel_case_types)]
    Operate_4,
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

#[derive(PartialEq, Copy, Clone)]
pub enum Event {
    #[allow(non_camel_case_types)]
    DL_SetMode_INACTIVE,
    #[allow(non_camel_case_types)]
    DL_SetMode_STARTUP,
    //TODO: add more
}

#[derive(Copy, Clone)]
pub enum EventError {
    InvalidState(State, Event),
}

pub trait StateActions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn wait_ms(&self, duration: u64);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn wait_on_event(&self);
}

pub struct StateMachine<T: StateActions> {
    state: State,
    state_actions: T,
    //retry: u8,
    event: Option<Event>,
    last_event_parse_result: Result<(), EventError>
}

impl<T: StateActions> StateMachine<T> {
    pub fn new(state_actions: T) -> Self {
        StateMachine {
            state: State::Idle_0,
            state_actions,
            //retry: 0,
            event: None, //TODO: i dont want flags
            last_event_parse_result: Ok(()), //TODO: i dont want flags
        }
    }

    pub async fn parse_event(&mut self, event: Event) -> Result<(), EventError> {
        self.event = Some(event);
        while self.event.is_some() {
            self.state_actions.wait_ms(1).await;
        }
        self.last_event_parse_result
    }

    // async fn wait_for_event(&mut self) -> Event {
    //     loop {
    //         while self.event.is_none() {
    //             self.state_actions.wait_ms(1).await;
    //         }
    //         self.last_event_parse_result = match self.event {
    //             Some(event) if self.state == State::Idle_0    && event == Event::DL_SetMode_STARTUP  => Ok(()),
    //             Some(event) if self.state == State::Operate_4 && event == Event::DL_SetMode_INACTIVE => Ok(()),
    //             Some(event) => Err(EventError::InvalidState(self.state, event)),
    //             None => panic!("This should never ever happen!"),
    //         };
    //         if self.last_event_parse_result.is_ok() {
    //             return self.event.take().unwrap()
    //         }
    //     }
    // }

    pub async fn run(&mut self) -> ! {
        info!("run state machine dlllllllllll");
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        match self.state {
            State::Idle_0 => {
                //let _ = self.wait_for_event().await;
                self.state_actions.wait_on_event().await;
                info!("enter State::EstablishCom_1");
                self.state = State::EstablishCom_1;
            },
            _ => {}
        }
    }
}
