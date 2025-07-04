//! Services of the Standardized Master Interface
//!
//! see [#11.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=175)

use structure_of_smi_service_arguments::*;
use annex_e::{ArgBlockID, ArgBlock, PortConfigList, JobError};

pub struct SmiResult<T: ArgBlock> {
    #[allow(dead_code)] //TODO: remove
    client_id: ClientID,
    #[allow(dead_code)] //TODO: remove
    port_number: PortNumber,
    #[allow(dead_code)] //TODO: remove
    ref_arg_block_id: RefArgBlockID,
    #[allow(dead_code)] //TODO: remove
    arg_block_length: ArgBlockLength,
    #[allow(dead_code)] //TODO: remove
    arg_block: T,
}

#[allow(dead_code)] // TODO: remove
#[allow(non_snake_case)]
pub fn SMI_PortConfiguration(
    _client_id: ClientID,
    _port_number: PortNumber,
    _arg_block_length: ArgBlockLength,
    _arg_block: PortConfigList,
) -> Result<SmiResult<PortConfigList>, SmiResult<JobError>> {
    const _EXP_ARG_BLOCK_ID: ExpArgBlockID = ArgBlockID::VoidBlock as ExpArgBlockID;
    Ok(SmiResult {
        client_id: 0,
        port_number: 0,
        ref_arg_block_id: 0,
        arg_block_length: 0,
        arg_block: PortConfigList {
            port_mode: 0,
            validation_and_backup: 0,
            iq_behavior: 0,
            port_cycle_time: 0,
            vendor_id: 0,
            device_id: 0,
        }
    })
}

/// see [#11.2.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=176)
mod structure_of_smi_service_arguments {
    pub type ClientID = u8;
    pub type PortNumber = u8;
    pub type ExpArgBlockID = u16;
    pub type RefArgBlockID = u16;
    pub type ArgBlockLength = u16;
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
        const A: () = assert!(Length <= 232, "Length must not exceed 232");
    }
}