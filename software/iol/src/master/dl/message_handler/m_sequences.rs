//! M-sequences
//!
//! see [#7.3.3.2 - IO-Link Specification](../../../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=81)

pub use crate::common::annex_a::CommunicationChannel;
use crate::common::annex_a::{self, RW};

#[allow(non_camel_case_types)]
#[allow(unused)] // TODO: remove
enum M_Sequence_Type{
    #[allow(non_camel_case_types)]
    TYPE_0,
    // TYPE_1_1,
    // TYPE_1_2,
    // TYPE_1_V,
    // TYPE_2_1,
    // TYPE_2_2,
    // TYPE_2_3,
    // TYPE_2_4,
    // TYPE_2_5,
    // TYPE_2_V,
}

// TODO: are all settings supported for all M-Sequence-Types?
#[allow(non_camel_case_types)]
pub struct M_Sequence<const SEND_ON_REQUEST_DATA: bool, const M: usize, const D: usize> {
    pub master_message: [u8; M],
    pub answer_length: usize,
}

impl<const SEND_ON_REQUEST_DATA: bool, const M: usize, const D: usize> M_Sequence<SEND_ON_REQUEST_DATA, M, D> {
    // pub const ANSWER_LENGTH: usize = D;
}

#[allow(non_camel_case_types)]
pub type TYPE_0<const SEND_ON_REQUEST_DATA: bool> = M_Sequence<SEND_ON_REQUEST_DATA, 2, 2>;
// #[allow(non_camel_case_types)]
// pub type TYPE_1_1<const SEND_ON_REQUEST_DATA: bool> = M_Sequence<SEND_ON_REQUEST_DATA, 4, 3>;

impl TYPE_0<false> {
    pub fn new(communication_channel: CommunicationChannel, address: u8) -> Self {
        let mut master_message = [
            annex_a::create_mc(RW::ReadAccess, communication_channel, address),
            annex_a::create_ckt_without_checksum(annex_a::M_Sequence_Type::Type_0),
        ];
        annex_a::calculate_checksum(&mut master_message);

        Self {
            master_message,
            answer_length: 2, //TODO: here D should be used somehow
        }
    }
}
