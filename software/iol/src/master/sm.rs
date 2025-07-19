//! System Management
//!
//! see [#9.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=122)

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

mod sm_services;
pub use sm_services::outside_sm as services;
pub use sm_services::{Service, ServiceResult};
pub use sm_services::sm_set_port_config;
use sm_services::inside_sm::*;

use crate::master::dl;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum State {
    #[allow(non_camel_case_types)]
    PortInactive_0,
    #[allow(non_camel_case_types)]
    CheckCompatibility_1,
    #[allow(non_camel_case_types)]
    DIDO_8,
    // JoinPseudoState_9 is not used. Instead an additional state to await DL_Mode_STARTUP is added between state 0 and 1
    AwaitStartup,

    // there is more
}

pub struct SM {
    state: State,
    comp_retry: u8,
}

impl SM {
    pub fn new() -> Self {
        Self {
            state: State::PortInactive_0,
            comp_retry: 0,
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        info!("{:?}", self.state);
        match &self.state {
            State::PortInactive_0 => {
                match receive_service().await {
                    Service::SM_SetPortConfig {target_mode, ..} => { // TODO: use all parameters
                        match target_mode {
                            sm_set_port_config::TargetMode::CFGCOM |
                            sm_set_port_config::TargetMode::AUTOCOM => {
                                dl::services::send_service(dl::Service::DL_SetMode {
                                    mode: dl::dl_set_mode::Mode::Startup,
                                    value_list: dl::dl_set_mode::ValueList {
                                        m_sequence_time: 0,
                                        m_sequence_type: dl::dl_set_mode::MSequenceType::TYPE_0,
                                        pd_input_length: 0,
                                        pd_output_length: 0,
                                        on_req_data_length_per_message: 0,
                                    }
                                }).await;
                                self.state = State::AwaitStartup;
                            }
                            sm_set_port_config::TargetMode::INACTIVE => self.state = State::PortInactive_0,
                            sm_set_port_config::TargetMode::DI |
                            sm_set_port_config::TargetMode::DO => self.state = State::DIDO_8,
                        }
                    }
                }
            },
            State::AwaitStartup => {
                let service = dl::services::receive_service().await;
                match service {
                    dl::Service::DL_Mode(real_mode) => {
                        match real_mode {
                            dl::dl_mode::RealMode::INACTIVE => {
                                // I think this makes sense, although this is not in the spec.
                                self.state = State::PortInactive_0;
                            }
                            dl::dl_mode::RealMode::COM2 |
                            dl::dl_mode::RealMode::COM3 => {
                                // do nothing for the moment
                                // maybe this mode can here be ignored for good
                            }
                            dl::dl_mode::RealMode::STARTUP => {
                                self.comp_retry = 0;
                                self.state = State::CheckCompatibility_1;
                            }
                            _ => panic!("unexpected mode: {:?}", real_mode),
                        }
                    }
                    _ => panic!("unexpected service: {:?}", service)
                }
            },
            State::CheckCompatibility_1 => {
                let _ = receive_service().await; // TODO: Just a dummy await here for now to keep the system running.
            },
            State::DIDO_8 => {
                let _ = receive_service().await; // TODO: Just a dummy await here for now to keep the system running.
            },
        }

    }
}
