//! System Management
//!
//! see [#9.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=122)

// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

use crate::master::dl;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub enum InspectionLevel {
    #[allow(non_camel_case_types)]
    NO_CHECK,
    #[allow(non_camel_case_types)]
    TYPE_COMP,
    // IDENTICAL, // not recommended for new developments
}

#[derive(Copy, Clone)]
pub enum TargetMode {
    CFGCOM,
    AUTOCOM,
    INACTIVE,
    DI,
    DO,
}

pub enum ErrorInfo {
    PARAMETER_CONFLICT,
}

pub enum Service {
    #[allow(non_camel_case_types)]
    SM_SetPortConfig {
        port_number: u8,
        configured_cycle_time: u8, //TODO: not clear what this is. is this the PortCycleTime from the SMI? data type?
        target_mode: TargetMode,
        configured_revision_id: u8,
        inspection_level: InspectionLevel,
        configured_vendor_id: u16,
        configured_device_id: u32, // Note: In the SMI it is defined as u32 with 3 octets used.
        configured_function_id: u16,
        configured_serial_number: u8 // TODO: specification says: up to 16 octets (see Table 80). Don't know yet what we need.
    },
}

pub struct PortConfigSuccess {
    pub port_number: u8,
}

pub struct PortConfigFail {
    pub port_number: u8
}
impl PortConfigFail {
    const ERROR_INFO: ErrorInfo = ErrorInfo::PARAMETER_CONFLICT;
}

pub enum ServiceResult {
    #[allow(non_camel_case_types)]
    SM_SetPortConfig(Result<PortConfigSuccess, PortConfigFail>),
}

pub static SERVICE_CHANNEL: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();


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
        match &self.state {
            State::PortInactive_0 => {
                match SERVICE_CHANNEL.receive().await {
                    Service::SM_SetPortConfig {target_mode, ..} => { // TODO: use all parameters
                        match target_mode {
                            TargetMode::CFGCOM | TargetMode::AUTOCOM => self.state = State::AwaitStartup,
                            TargetMode::INACTIVE => self.state = State::PortInactive_0,
                            TargetMode::DI | TargetMode::DO => self.state = State::DIDO_8,
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
                let _ = SERVICE_CHANNEL.receive().await; // TODO: Just a dummy await here for now to keep the system running.
            },
            State::DIDO_8 => {
                let _ = SERVICE_CHANNEL.receive().await; // TODO: Just a dummy await here for now to keep the system running.
            },
            // State::JoinPseudoState_9(service) => {
            //     match service {
            //         Service::SM_SetPortConfig {target_mode, ..} => { // TODO: use the paramaters
            //             match target_mode {
            //                 TargetMode::INACTIVE => {
            //                     dl::services::send_service(dl::Service::DL_SetMode {
            //                         mode: dl::dl_setmode::Mode::Inactive,
            //                         value_list: dl::dl_setmode::ValueList { // TODO: is a value list necessary for Inactive?
            //                             m_sequence_time: 0,
            //                             m_sequence_type: dl::dl_setmode::MSequenceType::TYPE_0,
            //                             pd_input_length: 0,
            //                             pd_output_length: 0,
            //                             on_req_data_length_per_message: 0,
            //                         }
            //                     }).await;

            //                     // TODO: PL_SetMode SDCI

            //                     self.state = State::PortInactive_0;
            //                 }
            //                 TargetMode::CFGCOM | TargetMode::AUTOCOM => {
            //                     self.state = State::PortInactive_0;
            //                 }
            //                 TargetMode::DI | TargetMode::DO => {
            //                     self.state = State::DIDO_8
            //                 }
            //             }
            //         },
            //         _ => panic!("unexpected service"),
            //     }
            // },
        }

    }
}
