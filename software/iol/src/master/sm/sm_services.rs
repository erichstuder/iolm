//! SM Master services
//!
//! see
//! - [#9.2.2- IO-Link Specification](../../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=124)

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub mod outside_sm {
    pub async fn send_service(service: super::Service) {
        super::SERVICE_TO_SM.send(service).await;
    }

    pub async fn receive_service_result() -> super::ServiceResult {
        super::RESULT_FROM_SM.receive().await
    }

//     pub async fn receive_service() -> super::Service {
//         super::SERVICE_FROM_SM.receive().await
//     }

//     pub async fn send_service_result(result: super::ServiceResult) {
//         super::RESULT_TO_SM.send(result).await
//     }
}

pub mod inside_sm {
//     pub async fn send_service(service: super::Service) {
//         super::SERVICE_FROM_SM.send(service).await;
//     }

//     pub async fn receive_service_result() -> super::ServiceResult {
//         super::RESULT_TO_SM.receive().await
//     }

    pub async fn receive_service() -> super::Service {
        super::SERVICE_TO_SM.receive().await
    }

//     pub async fn send_service_result(result: super::ServiceResult) {
//         super::RESULT_FROM_SM.send(result).await;
//     }
}

static SERVICE_TO_SM: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
static RESULT_FROM_SM: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();

// static SERVICE_FROM_SM: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
// static RESULT_TO_SM: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();

pub enum Service {
    #[allow(non_camel_case_types)]
    #[allow(unused)] // TODO: remove
    SM_SetPortConfig {
        port_number: u8,
        configured_cycle_time: u8, //TODO: not clear what this is. is this the PortCycleTime from the SMI? data type?
        target_mode: sm_set_port_config::TargetMode,
        configured_revision_id: u8,
        inspection_level: sm_set_port_config::InspectionLevel,
        configured_vendor_id: u16,
        configured_device_id: u32, // Note: In the SMI it is defined as u32 with 3 octets used.
        configured_function_id: u16,
        configured_serial_number: u8 // TODO: specification says: up to 16 octets (see Table 80). Don't know yet what we need.
    },
}

pub enum ServiceResult {
    #[allow(non_camel_case_types)]
    #[allow(unused)] // TODO: remove
    SM_SetPortConfig(Result<sm_set_port_config::Success, sm_set_port_config::Fail>),
}

pub mod sm_set_port_config {
    #[allow(unused)] // TODO: remove
    pub enum TargetMode {
        CFGCOM,
        AUTOCOM,
        INACTIVE,
        DI,
        DO,
    }

    pub enum InspectionLevel {
        #[allow(non_camel_case_types)]
        NO_CHECK,
        #[allow(non_camel_case_types)]
        #[allow(unused)] // TODO: remove
        TYPE_COMP,
        // IDENTICAL, // not recommended for new developments
    }

    pub struct Success {
        pub port_number: u8, // TODO: correct data type?
    }

    #[allow(unused)] // TODO: remove
    pub enum ErrorInfo {
        #[allow(non_camel_case_types)] //TODO: is this necessary?
        PARAMETER_CONFLICT,
    }

    pub struct Fail {
        pub port_number: u8, // TODO: correct data type?
    }
    impl Fail {
        #[allow(unused)] // TODO: remove
        const ERROR_INFO: ErrorInfo = ErrorInfo::PARAMETER_CONFLICT;
    }
}
