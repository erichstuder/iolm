// see #7.3.3

// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

enum State {
    #[allow(non_camel_case_types)]
    Inactive_0,
    #[allow(non_camel_case_types)]
    AwaitReply_1,
    #[allow(non_camel_case_types)]
    Startup_2,
    // Response_3,
    // AwaitReply_4,
    // ErrorHandling_5,
    // Preoperate_6,
    // GetOD_7,
    // Resonse_8,
    // AwaitReply_9,
    // ErrorHandling_10,
    // CheckHandler_11,
    // Operate_12,
    // GetPD_13,
    // GetOD_14,
    // Response_15,
    // AwaitReply_16,
    // ErrorHandling_17,
}

// TODO: Is this the right place for this enum? see also Table 9
pub enum TransmissionRate {
    COM1 = 4800,
    COM2 = 38400,
    COM3 = 230400,
}

pub enum Event {
    MH_Conf_COMx(TransmissionRate),
}

// pub trait Actions {
//     fn 
// }

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

pub struct StateMachine {
    state: State,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: State::Inactive_0,
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

    async fn confirm_event(&self) {
        RESULT_CHANNEL.send(()).await;
    }

    async fn next(&mut self) {
        match self.state {
            State::Inactive_0 => {
                let event = self.await_event().await;
                match event {
                    Event::MH_Conf_COMx(rate) => {
                        // send message
                    }
                }
            },
            State::AwaitReply_1 => {
                self.confirm_event().await;
                // if timeout or response not ok
                // self.state = State::Inactive_0;

                // if Response OK
                // self.state = State::Startup_2;
            },
            State::Startup_2 => {

            }
        }
    }
}
