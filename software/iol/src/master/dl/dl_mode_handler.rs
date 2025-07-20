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
use crate::master::dl::message_handler as mh;
use crate::master::pl;
use crate::master::pl::dynamic_characteristic_of_the_transmission as com_properties;

use wake_up_procedure_and_retry_characteristics as wake_up_properties;
use core::time::Duration;

#[derive(Debug, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    ComRequestCOM3_6,
    ComRequestCOM2_7,
    ComRequestCOM1_8,
    #[allow(non_camel_case_types)]
    Retry_9,
    #[cfg(feature = "iols")]
    #[allow(non_camel_case_types)]
    WaitOnReadyPulse_10,
    #[cfg(feature = "iols")]
    #[allow(non_camel_case_types)]
    WaitOnPortPowerOn_11,
}

pub enum ReadyPulseResult {
    ReadyPulseOk,
    // Note: It is more elegant if TimeToReadyElapsed is also an Event instead of a Guard.
    TimeToReadyElapsed,
}

#[cfg(feature = "iols")]
#[derive(PartialEq)]
enum Safety {
    #[allow(dead_code)] //TODO: remove as soon as NonSafety is assigned
    NonSafety,
    SafetyCom,
}

pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait(&self, duration: Duration);

    #[allow(async_fn_in_trait)]
    async fn await_ready_pulse_with_timeout(&self, duration: Duration) -> ReadyPulseResult;

    #[allow(async_fn_in_trait)]
    async fn port_power_off_on(&self, duration: Duration);
}

pub struct StateMachine<A> {
    state: State,
    actions: A,
    retry: u8,
    #[cfg(feature = "iols")]
    safety: Safety,
    #[cfg(feature = "iols")]
    min_shutdown_time: Duration,
    #[cfg(feature = "iols")]
    time_to_ready: Duration,
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
            min_shutdown_time: Duration::from_millis(3000), //TODO: don't know yet where it will be set from.
            #[cfg(feature = "iols")]
            time_to_ready: Duration::from_millis(5000), //TODO: don't know yet where it will be set from
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        info!("{:?}", self.state);
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
                self.actions.port_power_off_on(self.min_shutdown_time).await;
                self.state = State::WaitOnReadyPulse_10;
            },
            #[cfg(feature = "iols")]
            State::WaitOnReadyPulse_10 => {
                match self.actions.await_ready_pulse_with_timeout(self.time_to_ready).await {
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
            },
            State::EstablishCom_1 => {
                self.state = State::WURQ_5;
            },
            State::WURQ_5 => {
                pl::services::send_service(pl::Service::PL_WakeUp).await;
                let result = pl::services::receive_service_result().await;
                if result != pl::ServiceResult::PL_WakeUp {
                    panic!("unexpected result: {:?}", result);
                }
                self.state = State::ComRequestCOM3_6;
            },
            State::ComRequestCOM3_6 => {
                self.actions.wait(wake_up_properties::com3::T_DMT).await;

                // Note: There is some confusion.
                // On the one hand it says: "Set transmission rate ..." in T15 to T17 of DL-mode handler.
                // On the other hand T1 of the message handler says: "Send a message with the requested transmission rate ...".
                // So it is not clear whether to set the transmission rate here or in message handler.
                // As it feels cleaner the transmission rate is set there.

                mh::EVENT_CHANNEL.send(mh::Event::MH_Conf_COMx {
                    transmission_rate: com_properties::com3::F_DTR,
                }).await;
                mh::RESULT_CHANNEL.receive().await;

                // TODO: we jump right to COM2 => fix
                self.state = State::ComRequestCOM2_7;

                // if success then:
                // send_service(Service::DL_Mode(dl_mode::RealMode::COM3)).await;
                // send_service(Service::DL_Mode(dl_mode::RealMode::STARTUP)).await;
            },
            State::ComRequestCOM2_7 => {
                info!("1");
                self.actions.wait(wake_up_properties::com2::T_DMT).await;
                info!("2");
                mh::EVENT_CHANNEL.send(mh::Event::MH_Conf_COMx {
                    transmission_rate: com_properties::com2::F_DTR,
                }).await;
                info!("3");
                mh::RESULT_CHANNEL.receive().await;
                info!("4");

                self.state = State::ComRequestCOM1_8; // TODO: implement the other exit

                self.actions.wait(Duration::from_secs(100)).await;// dummy wait

                // if success then:
                // send_service(Service::DL_Mode(dl_mode::RealMode::COM2)).await;
                // send_service(Service::DL_Mode(dl_mode::RealMode::STARTUP)).await;
            }
            State::ComRequestCOM1_8 => {
                self.actions.wait(wake_up_properties::com1::T_DMT).await;

                self.actions.wait(Duration::from_secs(10)).await; // dummy wait

                self.state = State::Retry_9; // dummy state change

                // if success then:
                // send_service(Service::DL_Mode(dl_mode::RealMode::COM1)).await;
                // send_service(Service::DL_Mode(dl_mode::RealMode::STARTUP)).await;
            },
            State::Retry_9 => {
                self.actions.wait(Duration::from_secs(10)).await; // dummy wait
            },
        }
    }
}

// see Table 42
mod wake_up_procedure_and_retry_characteristics {
    use crate::master::pl::dynamic_characteristic_of_the_transmission as com_properties;
    use core::time::Duration;

    const fn multiply(factor: u8, duration: Duration) -> Duration {
        Duration::from_nanos(((factor as u128) * duration.as_nanos()) as u64)
    }

    pub mod com1{
        use super::*;
        pub const T_DMT: Duration = multiply(32, com_properties::com1::T_BIT);
    }

    pub mod com2{
        use super::*;
        pub const T_DMT: Duration = multiply(32, com_properties::com2::T_BIT);
    }

    pub mod com3{
        use super::*;
        pub const T_DMT: Duration = multiply(32, com_properties::com3::T_BIT);
    }
}
