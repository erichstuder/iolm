//! Services of the Standardized Master Interface
//!
//! see [#11.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=175)

use crate::master::sm;
use structure_of_smi_service_arguments::*;
use annex_e::{ArgBlockID, ArgBlock, PortConfigList, VoidBlock, JobError};

pub struct SmiResult<T: ArgBlock> {
    client_id: ClientID,
    port_number: PortNumber,
    ref_arg_block_id: RefArgBlockID,
    //arg_block_length: ArgBlockLength, // This value is not needed in this implementation as we work with structs.
    arg_block: T,
}

#[allow(non_snake_case)]
/// SMI Port Configuration service
///
/// Note: Argument arg_block_length has been left out. If is not necessary in this implementation as we work with structs.
pub async fn SMI_PortConfiguration(
    client_id: ClientID,
    port_number: PortNumber,
    arg_block: PortConfigList,
) -> Result<SmiResult<VoidBlock>, SmiResult<JobError>> {
    const _EXP_ARG_BLOCK_ID: ExpArgBlockID = ArgBlockID::VoidBlock as ExpArgBlockID;

    let revision_id: u8 = 0x11; // Note: According to B.1.5 this can be overwritten. Where? By who?

    let port_config = sm::Service::SM_SetPortConfig {
        port_number,
        configured_cycle_time: arg_block.port_cycle_time,
        target_mode: sm::sm_set_port_config::TargetMode::INACTIVE, // TODO: set correct value
        configured_revision_id: revision_id,
        inspection_level: sm::sm_set_port_config::InspectionLevel::NO_CHECK, //TODO: set correct value
        configured_vendor_id: arg_block.vendor_id,
        configured_device_id: arg_block.device_id,
        configured_function_id: 0x0000, // TODO: don't know yet what this is for
        configured_serial_number: 0x00, // TODO: will be implemented later
    };
    sm::services::send_service(port_config).await;

    let service_result = sm::services::receive_service_result().await;
    match service_result {
        sm::ServiceResult::SM_SetPortConfig(result) => {
            match result {
                Ok(data) => {
                    Ok(SmiResult {
                        client_id,
                        port_number: data.port_number,
                        ref_arg_block_id: arg_block.get_arg_block_id(),
                        arg_block: VoidBlock,
                    })
                }
                Err(data) => {
                    Err(SmiResult {
                        client_id,
                        port_number: data.port_number,
                        ref_arg_block_id: arg_block.get_arg_block_id(),
                        arg_block: JobError {
                            exp_arg_block_id: VoidBlock::ARG_BLOCK_ID,
                            error_code: 0, // TODO: set correct code
                            additional_code: 0, // TODO: set correct code
                        }
                    })
                }
            }
        }
    }
}

/// see [#11.2.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=176)
mod structure_of_smi_service_arguments {
    pub type ClientID = u8;
    pub type PortNumber = u8;
    pub type ExpArgBlockID = u16;
    pub type RefArgBlockID = u16;
    // pub type ArgBlockLength = u16; // This value is not needed in this implementation as we work with structs.
    // ArgBlock is variable
}

/// see [#Annex E - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=278)
mod annex_e {

    use super::annex_f::OctetStringT;

    /// see [#E.1 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=278)
    pub enum ArgBlockID {
        // MasterIdent        = 0x0001,
        // FSMasterAccess     = 0x0100,
        // WMasterConfig      = 0x0200,
        // PDIn               = 0x1001,
        // PDOut              = 0x1002,
        // PDInOut            = 0x1003,
        // SPDUIn             = 0x1101,
        // SPDUOut            = 0x1102,
        // PDInIQ             = 0x1FFE,
        // PDOutIQ            = 0x1FFF,
        // OnRequestDataWrite = 0x3000,
        // OnRequestDataRead  = 0x3001,
        // #[allow(non_camel_case_types)]
        // DS_Data            = 0x7000,
        // DeviceParBatch     = 0x7001,
        // IndexList          = 0x7002,
        // PortPowerOffOn     = 0x7003,
        // PortConfigList     = 0x8000,
        // FSPortConfigList   = 0x8100,
        // WTrackConfigList   = 0x8200,
        // PortStatusList     = 0x9000,
        // FSPortStatusList   = 0x9100,
        // WTrackStatusList   = 0x9200,
        // WTrackScanResult   = 0x9201,
        // DeviceEvent        = 0xA000,
        // PortEvent          = 0xA001,
        VoidBlock        = 0xFFF0,
        // JobError           = 0xFFFF,
    }

    pub trait ArgBlock {
        const ARG_BLOCK_ID: u16;

        fn get_arg_block_id(&self) -> u16 {
            Self::ARG_BLOCK_ID
        }
    }

    pub struct MasterIdent<const MAX_NUMBER_OF_PORTS: usize> {
        vendor_id: u16,
        master_id: u32,
        master_type: u8,
        features_1: u8,
        features_2: u8,
        max_number_of_ports: u8,
        port_types: [u8; MAX_NUMBER_OF_PORTS],
    }
    impl<const MAX_NUMBER_OF_PORTS: usize> ArgBlock for MasterIdent<MAX_NUMBER_OF_PORTS> {
        const ARG_BLOCK_ID: u16 = 0x0001;
    }
    impl<const MAX_NUMBER_OF_PORTS: usize> MasterIdent<MAX_NUMBER_OF_PORTS> {
        const MAX_NUMBER_OF_PORTS: u8 = MAX_NUMBER_OF_PORTS as u8;
    }

    pub struct PortConfigList {
        pub port_mode: u8,
        pub validation_and_backup: u8,
        pub iq_behavior: u8,
        pub port_cycle_time: u8,
        pub vendor_id: u16,
        pub device_id: u32,
    }
    impl ArgBlock for PortConfigList {
        const ARG_BLOCK_ID: u16 = 0x8000;
    }

    pub struct OnRequestDataWrite<const DATA_LEN: usize> {
        pub index: u16,
        pub subindex: u8,
        pub on_request_data: OctetStringT<DATA_LEN>,
    }
    impl<const DATA_LEN: usize> OnRequestDataWrite<DATA_LEN> {
        const ARG_BLOCK_ID: u16 = 0x3000;
    }

    pub struct OnRequestDataRead<const DATA_LEN: usize> {
        pub index: u16,
        pub subindex: u8,
        pub on_request_data: OctetStringT<DATA_LEN>,
    }
    impl<const DATA_LEN: usize> OnRequestDataRead<DATA_LEN> {
        const ARG_BLOCK_ID: u16 = 0x3001;
    }

    pub struct VoidBlock;
    impl ArgBlock for VoidBlock {
        const ARG_BLOCK_ID: u16 = 0xFFF0;
    }

    pub struct JobError {
        pub exp_arg_block_id: u16,
        pub error_code: u8,
        pub additional_code: u8,
    }
    impl ArgBlock for JobError {
        const ARG_BLOCK_ID: u16 = 0xFFFF;
    }
}

mod annex_f {
    pub struct OctetStringT<const Length: usize> {
        pub sequence: [u8; Length],
    }
    impl<const Length: usize> OctetStringT<Length> {
        const _A: () = assert!(Length <= 232, "Length must not exceed 232");
    }
}