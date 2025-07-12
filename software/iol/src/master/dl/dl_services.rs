//! Data link layer services
//!
//! see
//! - [#7.2- IO-Link Specification](../../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=58)

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub mod outside_dl {
    pub async fn send_service(service: super::Service) {
        super::SERVICE_TO_DL.send(service).await;
    }

    pub async fn receive_service_result() -> super::ServiceResult {
        super::RESULT_FROM_DL.receive().await
    }

    pub async fn receive_service() -> super::Service {
        super::SERVICE_FROM_DL.receive().await
    }

    pub async fn send_service_result(result: super::ServiceResult) {
        super::RESULT_TO_DL.send(result).await
    }
}

pub mod inside_dl {
    pub async fn send_service(service: super::Service) {
        super::SERVICE_FROM_DL.send(service).await;
    }

    pub async fn receive_service_result() -> super::ServiceResult {
        super::RESULT_TO_DL.receive().await
    }

    pub async fn receive_service() -> super::Service {
        super::SERVICE_TO_DL.receive().await
    }

    pub async fn send_service_result(result: super::ServiceResult) {
        super::RESULT_FROM_DL.send(result).await;
    }
}

static SERVICE_TO_DL: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
static RESULT_FROM_DL: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();

static SERVICE_FROM_DL: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
static RESULT_TO_DL: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();


#[derive(Debug)]
pub enum Service {
    #[allow(non_camel_case_types)]
    DL_SetMode {
        mode: dl_set_mode::Mode,
        value_list: dl_set_mode::ValueList,
    },
    #[allow(non_camel_case_types)]
    DL_Mode(dl_mode::RealMode),
}

pub enum ServiceResult {
    #[allow(non_camel_case_types)]
    DL_SetMode(Result<(), dl_set_mode::Fail>),
}

pub mod dl_set_mode {
    #[derive(Debug)]
    pub enum Mode {
        Inactive,
        Startup,
        Preoperate,
        Operate,
    }

    #[derive(Debug)]
    pub enum MSequenceType {
        #[allow(non_camel_case_types)]
        TYPE_0,
        #[allow(non_camel_case_types)]
        TYPE_1_1,
        #[allow(non_camel_case_types)]
        TYPE_1_2,
        #[allow(non_camel_case_types)]
        TYPE_1_V,
        #[allow(non_camel_case_types)]
        TYPE_2_1,
        #[allow(non_camel_case_types)]
        TYPE_2_2,
        #[allow(non_camel_case_types)]
        TYPE_2_3,
        #[allow(non_camel_case_types)]
        TYPE_2_4,
        #[allow(non_camel_case_types)]
        TYPE_2_5,
        #[allow(non_camel_case_types)]
        TYPE_2_V,
    }


    #[derive(Debug)]
    pub struct ValueList {
        pub m_sequence_time: u8, //TODO: correct data type?
        pub m_sequence_type: MSequenceType,
        pub pd_input_length: u8, //TODO: correct data type?
        pub pd_output_length: u8, //TODO: correct data type?
        pub on_req_data_length_per_message: u8, //TODO: correct data type?
    }

    pub enum ErrorInfo {
        StateConflict,
        ParameterConflict,
    }

    pub struct Fail {
        pub error_info: ErrorInfo,
    }
}

pub mod dl_mode {
    #[derive(Debug)]
    pub enum RealMode {
        INACTIVE,
        COM1,
        COM2,
        COM3,
        COMLOST,
        #[cfg(feature = "device")] // Note: Could find its use only for device.
        ESTABCOM,
        STARTUP,
        PREOPERATE,
        OPERATE,
    }
}
