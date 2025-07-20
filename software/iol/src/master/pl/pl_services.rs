//! Data link layer services
//!
//! see
//! - [#5.2- IO-Link Specification](../../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=42)

use embassy_sync::channel::{Channel, TryReceiveError};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub mod outside_pl {
    pub async fn send_service(service: super::Service) {
        super::SERVICE_TO_PL.send(service).await;
    }

    pub async fn receive_service_result() -> super::ServiceResult {
        super::RESULT_FROM_PL.receive().await
    }

    pub fn service_result_is_ready() -> bool {
        !super::RESULT_FROM_PL.is_empty()
    }
}

pub mod inside_dl {
    pub async fn receive_service() -> super::Service {
        super::SERVICE_TO_PL.receive().await
    }

    pub async fn send_service_result(result: super::ServiceResult) {
        super::RESULT_FROM_PL.send(result).await;
    }
}

static SERVICE_TO_PL: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
static RESULT_FROM_PL: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();

pub enum Service {
    #[allow(non_camel_case_types)]
    PL_SetMode(pl_set_mode::TargetMode),
    #[allow(non_camel_case_types)]
    PL_WakeUp,
    #[allow(non_camel_case_types)]
    PL_Transfer {
        data: [u8; 32],
        data_length: usize,
        answer_length: usize
    },
}

#[derive(PartialEq, Debug)]
pub enum ServiceResult {
    #[allow(non_camel_case_types)]
    PL_WakeUp,
    #[allow(non_camel_case_types)]
    PL_Transfer(Result<[u8; 32], pl_transfer::Fail>),
}

pub mod pl_set_mode {
    pub enum TargetMode {
        #[allow(unused)] // TODO: remove
        INACTIVE,
        #[allow(unused)] // TODO: remove
        DI,
        #[allow(unused)] // TODO: remove
        DO,
        COM1,
        COM2,
        COM3,
    }
}

pub mod pl_transfer {
    #[derive(PartialEq, Debug)]
    pub enum Fail {
        #[allow(non_camel_case_types)]
        PARITY_ERROR,
        #[allow(non_camel_case_types)]
        FRAMING_ERROR,
        OVERRUN,
    }
}
