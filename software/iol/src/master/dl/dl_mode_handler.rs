//! State machine fo the Master DL-mode handler
//!
//! see
//! - [#7.3.2.4 - IO-Link Specification](../../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=75)
//! - [#7.2 - IO-Link Safety Extension](../../../spec/IO-Link_Safety_System-Extensions_10092_V114_Oct24.pdf#page=40)

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

use crate::master::dl::dl_services::{Service, ServiceResult};
use crate::master::dl::dl_services::inside_dl::*;
use crate::master::dl::dl_services::{dl_set_mode, dl_mode};
use crate::master::pl;
use crate::master::dl::message_handler as mh;

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
    #[allow(non_camel_case_types)]
    WURQ_5,
    // ComRequestCOM3_6,
    ComRequestCOM2_7,
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

// #[derive(Debug, PartialEq, Copy, Clone)]
// pub enum Service {
//     #[allow(non_camel_case_types)]
//     DL_SetMode_INACTIVE,
//     #[allow(non_camel_case_types)]
//     DL_SetMode_STARTUP,
//     #[allow(non_camel_case_types)]
//     DL_SetMODE_PREOPERATE,
//     #[allow(non_camel_case_types)]
//     DL_SetMODE_OPERATE,
// }

pub enum ReadyPulseResult {
    ReadyPulseOk,
    // Note: It is more elegant if TimeToReadyElapsed is also an Event instead of a Guard.
    TimeToReadyElapsed,
}

// #[derive(Debug, Copy, Clone)]
// pub enum EventError {
//     #[allow(unused)] //TODO: remove
//     InvalidState(State, Service),
// }

#[cfg(feature = "iols")]
#[derive(PartialEq)]
enum Safety {
    #[allow(dead_code)] //TODO: remove as soon as NonSafety is assigned
    NonSafety,
    SafetyCom,
}

pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait_ms(&self, duration: u64);

    #[allow(async_fn_in_trait)]
    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> ReadyPulseResult;

    #[allow(async_fn_in_trait)]
    async fn port_power_off_on_ms(&self, duration: u64);
}

pub struct StateMachine<A> {
    state: State,
    actions: A,
    retry: u8,
    #[cfg(feature = "iols")]
    safety: Safety,
    #[cfg(feature = "iols")]
    min_shutdown_time_ms: u64,
    #[cfg(feature = "iols")]
    time_to_ready_ms: u64,
}

impl<A: Actions> StateMachine<A> {
    pub fn new(actions: A) -> Self {
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

    // async fn await_event(&self) -> Service {
    //     SERVICE_REQ.receive().await
    // }

    // async fn confirm_event(&self, result: Result<(), EventError>) {
    //     SERVICE_CNF.send(result).await;
    // }

    async fn next(&mut self) {
        match self.state {
            State::Idle_0 => {
                let service = receive_service().await;
                match service {
                    Service::DL_SetMode { mode, value_list: _ } => { //TODO: use value_list
                        match mode {
                            dl_set_mode::Mode::Startup => send_service_result(ServiceResult::DL_SetMode(Ok(()))).await,
                            _ => {
                                let result = dl_set_mode::Fail{ error_info: dl_set_mode::ErrorInfo::StateConflict };
                                send_service_result(ServiceResult::DL_SetMode(Err(result))).await;
                            }
                        }
                    },
                    _ => panic!("invalid service"),
                }

                #[cfg(feature = "iols")]
                if self.safety == Safety::SafetyCom {
                    self.state = State::WaitOnPortPowerOn_11;
                    return;
                }
                self.retry = 0;
                self.state = State::EstablishCom_1;
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
            State::EstablishCom_1 => {
                info!("EstablishCom_1");
                self.state = State::WURQ_5;
            },
            State::WURQ_5 => {
                info!("WURQ_5");
                pl::SERVICE_CHANNEL.send(pl::Service::PL_WakeUp).await;
                let result = pl::RESULT_CHANNEL.receive().await;
                if result != pl::ServiceResult::PL_WakeUp {
                    panic!("unexpected result: {:?}", result);
                }
                send_service(Service::DL_Mode(dl_mode::RealMode::COM2)).await;
                self.state = State::ComRequestCOM2_7; // Note: For the moment we jump directly to COM2 instead of COM3 => fix!
            },
            State::ComRequestCOM2_7 => {
                info!("ComRequestCOM2_7");
                // TODO: T_DMT is 32 * T_BIT which results in about 833us for COM2. We try 1ms
                // TODO: Where to put the speed Definitions for COM3, COM2, COM1 ?
                const T_DMT: u64 = 1;
                self.actions.wait_ms(T_DMT).await;
                // ComRequest
                mh::EVENT_CHANNEL.send(mh::Event::MH_Conf_COMx(mh::TransmissionRate::COM2)).await;
                mh::RESULT_CHANNEL.receive().await;

                self.actions.wait_ms(10000).await;
            }
        }
    }
}
