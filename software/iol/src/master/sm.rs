//! System Management
//!
//! see [#9.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=122)

// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub enum InspectionLevel {
    #[allow(non_camel_case_types)]
    NO_CHECK,
    #[allow(non_camel_case_types)]
    TYPE_COMP,
    // IDENTICAL, // not recommended for new developments
}

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
    port_number: u8,
}

pub struct PortConfigFail {
    port_number: u8
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
    JoinPseudoState_9,
    // there is more
}

pub struct SM {
    state: State,
}

impl SM {
    pub fn new() -> Self {
        Self {
            state: State::PortInactive_0,
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn next(&mut self) {
        match self.state {
            State::PortInactive_0 => {
                // I think first we need to go to JoinPseudoState_9 then a variable will be set and we may take T1. But not yet clear.
                self.state = State::JoinPseudoState_9;
            },
            State::JoinPseudoState_9 => {
                match SERVICE_CHANNEL.receive().await {
                    Service::SM_SetPortConfig {..} => return, //TODO: implement
                }
            }
        }

    }
}
