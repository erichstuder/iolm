//! Services of the Standardized Master Interface
//!
//! see [#11.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=175)

use structure_of_smi_service_arguments::*;
use annex_e::{ArgBlockID_T, ArgBlockID};

#[allow(dead_code)] // TODO: remove
#[allow(non_snake_case)]
pub fn SMI_PortConfiguration(
    _client_id: ClientID,
    _port_number: PortNumber,
    _arg_block_length: ArgBlockLength,
    _arg_block: ArgBlockID_T,
) -> Result<(), ()> { // TODO: define Result
    let _exp_arg_block_id: ExpArgBlockID = ArgBlockID::VoidBlock as ExpArgBlockID;
    Ok(())
}

/// see [#11.2.2 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=176)
mod structure_of_smi_service_arguments {
    use super::annex_e::ArgBlockID_T;

    pub type ClientID = u8;
    pub type PortNumber = u8;
    pub type ExpArgBlockID = ArgBlockID_T;
    // pub type RefArgBlockID = ArgBlockID_T;
    pub type ArgBlockLength = u16;
    // ArgBlock is variable
}

/// see [#Annex E - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=278)
mod annex_e {

    #[allow(non_camel_case_types)]
    pub type ArgBlockID_T = u16; //TODO: rename, I dont like the _T

    /// see [#E.1 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=278)
    pub enum ArgBlockID {
        // MasterIdent      = 0x0001,
        // FSMasterAccess   = 0x0100,
        // WMasterConfig    = 0x0200,
        // PDIn             = 0x1001,
        // PDOut            = 0x1002,
        // PDInOut          = 0x1003,
        // SPDUIn           = 0x1101,
        // SPDUOut          = 0x1102,
        // PDInIQ           = 0x1FFE,
        // PDOutIQ          = 0x1FFF,
        // OnRequestData   = 0x3000, // would a better name be On_requestDataWrite?
        // _empty_          = 0x3001, // is this On_requestDataRead?
        // #[allow(non_camel_case_types)]
        // DS_Data          = 0x7000,
        // DeviceParBatch   = 0x7001,
        // IndexList        = 0x7002,
        // PortPowerOffOn   = 0x7003,
        // PortConfigList   = 0x8000,
        // FSPortConfigList = 0x8100,
        // WTrackConfigList = 0x8200,
        // PortStatusList   = 0x9000,
        // FSPortStatusList = 0x9100,
        // WTrackStatusList = 0x9200,
        // WTrackScanResult = 0x9201,
        // DeviceEvent      = 0xA000,
        // PortEvent        = 0xA001,
        VoidBlock        = 0xFFF0,
        // JobError         = 0xFFFF,
    }

    // pub mod ArgBlocks {
    //     pub struct VoidBlock;
    //     impl VoidBlock {
    //         const UniqueID: ArgBlockID = 0xFFF0;
    //     }
    //     //idea: more ArgBlocks with generics parameters.
    // }
}
