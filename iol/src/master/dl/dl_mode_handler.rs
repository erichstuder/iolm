#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

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
    #[cfg(feature = "iols")]
    #[allow(non_camel_case_types)]
    WaitOnReadyPulse_10,
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

pub enum ReadyPulseResult {
    ReadyPulseOk,
    // Note: It is more elegant if TimeToReadyElapsed is also an Event instead of a Guard.
    TimeToReadyElapsed,
}

#[derive(Debug, Copy, Clone)]
pub enum EventError {
    #[allow(unused)] //TODO: remove
    InvalidState(State, Event),
}

#[cfg(feature = "iols")]
#[derive(PartialEq)]
enum Safety {
    #[allow(dead_code)] //TODO: remove as soon as NonSafety is assigned
    NonSafety,
    SafetyCom,
}

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn wait_ms(&self, duration: u64);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> ReadyPulseResult;
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_off_on_ms(&self, duration: u64);
}

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, Result<(), EventError>, 1> = Channel::new();

pub struct StateMachine<T: Actions> {
    state: State,
    actions: T,
    retry: u8,
    #[cfg(feature = "iols")]
    safety: Safety,
    #[cfg(feature = "iols")]
    min_shutdown_time_ms: u64,
    #[cfg(feature = "iols")]
    time_to_ready_ms: u64,
}

impl<T: Actions> StateMachine<T> {
    pub fn new(actions: T) -> Self {
        Self {
            state: State::Idle_0,
            actions,
            retry: 0,
            #[cfg(feature = "iols")]
            safety: Safety::SafetyCom, //TODO: don't know yet where it will be set from.
            #[cfg(feature = "iols")]
            min_shutdown_time_ms: 3000, //TODO: don't know yet where it will be set from.
            #[cfg(feature = "iols")]
            time_to_ready_ms: 5000, //TODO: don't know yet where it will be set from
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn await_event(&self) -> Event {
        EVENT_CHANNEL.receive().await
    }

    async fn confirm_event(&self, result: Result<(), EventError>) {
        RESULT_CHANNEL.send(result).await;
    }

    async fn next(&mut self) {
        match self.state {
            State::Idle_0 => {
                let event = self.await_event().await;
                match event {
                    Event::DL_SetMode_STARTUP => self.confirm_event(Ok(())).await,
                    _ => self.confirm_event(Err(EventError::InvalidState(State::Idle_0, event))).await,
                }

                #[cfg(feature = "iols")]
                if self.safety == Safety::SafetyCom {
                    self.state = State::WaitOnPortPowerOn_11;
                    return;
                }
                self.retry = 0;
                self.state = State::EstablishCom_1;
            },
            State::EstablishCom_1 => {
                info!("EstablishCom_1");
                self.actions.wait_ms(3000).await;
            },
            #[cfg(feature = "iols")]
            State::WaitOnPortPowerOn_11 => {
                info!("WaitOnPortPowerOn_11");
                self.actions.port_power_off_on_ms(self.min_shutdown_time_ms).await;
                self.state = State::WaitOnReadyPulse_10;
                info!("done");
            },
            #[cfg(feature = "iols")]
            State::WaitOnReadyPulse_10 => {
                info!("WaitOnReadyPulse_10");
                match self.actions.await_ready_pulse_with_timeout_ms(self.time_to_ready_ms).await {
                    ReadyPulseResult::ReadyPulseOk => {
                        info!("ReadyPulseOk");
                        // Note:
                        // Strangely the specification wants to enter this state on DL_SetMode_STARTUP.
                        // To me this makes no sense. Or is there some magic I don't understand yet?
                        self.retry = 0;
                        self.state = State::EstablishCom_1;
                    },
                    ReadyPulseResult::TimeToReadyElapsed => {
                        info!("TimeToReadyElapsed");
                        self.state = State::Idle_0;
                    }
                }

                self.actions.wait_ms(1000).await; //TODO:remove
            },
        }
    }
}
