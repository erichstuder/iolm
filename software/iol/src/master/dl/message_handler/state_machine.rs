//! State machine of the Master message handler
//!
//! see [#7.3.3 - IO-Link Specification](../../../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=83)

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use crate::common::annex_b::direct_parameter_page_1_and_2::address;
use crate::master::dl::message_handler::m_sequences;
use crate::master::pl;
use crate::master::pl::dynamic_characteristic_of_the_transmission as com_properties;
use core::time::Duration;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum State {
    #[allow(non_camel_case_types)]
    Inactive_0,
    #[allow(non_camel_case_types)]
    #[allow(unused)] //TODO: remove
    AwaitReply_1,
    #[allow(non_camel_case_types)]
    #[allow(unused)] //TODO: remove
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

// // TODO: Is this the right place for this enum? see also Table 9
// pub enum TransmissionRate {
//     //COM1 = 4800,
//     COM2 = 38400,
//     //COM3 = 230400,
// }

pub enum Event {
    #[allow(non_camel_case_types)]
    MH_Conf_COMx {
        transmission_rate: u32,
    },
}

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait(&self, duration: Duration);
}

pub struct StateMachine<A> {
    state: State,
    actions: A,
}

impl<A: Actions> StateMachine<A> {
    pub fn new(actions: A) -> Self {
        Self {
            state: State::Inactive_0,
            actions,
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
        info!("{:?}", self.state);
        match self.state {
            State::Inactive_0 => {
                let event = self.await_event().await;
                match event {
                    Event::MH_Conf_COMx { transmission_rate } => {
                        match transmission_rate {
                            com_properties::com1::F_DTR => pl::services::send_service(pl::Service::PL_SetMode(pl::pl_set_mode::TargetMode::COM1)).await,
                            com_properties::com2::F_DTR => pl::services::send_service(pl::Service::PL_SetMode(pl::pl_set_mode::TargetMode::COM2)).await,
                            com_properties::com3::F_DTR => pl::services::send_service(pl::Service::PL_SetMode(pl::pl_set_mode::TargetMode::COM3)).await,
                            _ => panic!("unknown baudrate"),
                        }

                        let m_sequence = m_sequences::TYPE_0::new(m_sequences::CommunicationChannel::Page, address::MinCycleTime);

                        pl::services::send_service(
                            pl::Service::PL_Transfer {
                                data: {
                                    let mut buf = [0u8; 32];
                                    let msg = &m_sequence.master_message;
                                    buf[..msg.len()].copy_from_slice(msg);
                                    buf
                                },
                                data_length: m_sequence.master_message.len(),
                                answer_length: m_sequence.answer_length
                            }
                        ).await;
                        info!("wait on answer of test message");
                        self.actions.wait(Duration::from_millis(10)).await; //TODO: set right time and currently waiting the whole m-sequence time because it takes that long anyway.
                        match pl::services::service_result_is_ready() {
                            true => {
                                let _answer = pl::services::receive_service_result().await;
                                info!("answer on test message received");
                            }
                            false => {
                                info!("answer on test message timedouttttttttttttttttt");
                            }
                        }
                        // if let pl::ServiceResult::PL_Transfer { .. } = answer {
                        //     for item in answer {
                        //         info!("answer[{}]: {:?}", i, item);
                        //     }
                        // }
                        self.confirm_event().await;
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
