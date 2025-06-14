// see #7.3.3.2

pub use annex_a::{RW, CommunicationChannel};

#[allow(non_camel_case_types)]
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
    pub device_message: [u8; D],
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
        annex_a::calculate_checksum(&mut master_message, 2);

        Self {
            master_message,
            device_message: [0;2],
        }
    }
}

// TODO: move annex A to its own file?
mod annex_a {
    pub enum RW {
        WriteAccess = 0,
        ReadAccess = 1,
    }

    pub enum CommunicationChannel {
        Process = 0,
        Page = 1,
        Diagnosis = 2,
        ISDU = 3,
    }


    #[allow(non_camel_case_types)]
    pub enum M_Sequence_Type {
        #[allow(non_camel_case_types)]
        Type_0 = 0,
        #[allow(non_camel_case_types)]
        Type_1 = 1,
        #[allow(non_camel_case_types)]
        Type_2 = 2,
        // reserved = 3,
    }

    pub fn create_mc(rw: RW, channel: CommunicationChannel, address: u8) -> u8 {
        if address > 0b1_1111 {
            panic!("invalid address size");
        }
        ((rw as u8) << 7) | ((channel as u8) << 5) | (address)
    }

    pub fn create_ckt_without_checksum(m_sequence_type: M_Sequence_Type) -> u8 {
        (m_sequence_type as u8) << 6
    }

    pub fn calculate_checksum(message: &mut [u8], size: usize) {
        const SEED: u8 = 0x52;
        let mut result = SEED;
        for n in 0..size {
            result ^= message[n];
        }

        let d7_8 = (result >> 7) & 1;
        let d6_8 = (result >> 6) & 1;
        let d5_8 = (result >> 5) & 1;
        let d4_8 = (result >> 4) & 1;
        let d3_8 = (result >> 3) & 1;
        let d2_8 = (result >> 2) & 1;
        let d1_8 = (result >> 1) & 1;
        let d0_8 = (result >> 0) & 1;

        let d5_6 = d7_8 ^ d5_8 ^ d3_8 ^ d1_8;
        let d4_6 = d6_8 ^ d4_8 ^ d2_8 ^ d0_8;
        let d3_6 = d7_8 ^ d6_8;
        let d2_6 = d5_8 ^ d4_8;
        let d1_6 = d3_8 ^ d2_8;
        let d0_6 = d1_8 ^ d0_8;

        let checksum = 0x00 | (d5_6 << 5) | (d4_6 << 4) | (d3_6 << 3) | (d2_6 << 2) | (d1_6 << 1) | d0_6;
        message[1] |= checksum;
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_mc() {
            let mc: u8 = create_mc(RW::WriteAccess, CommunicationChannel::Process, 0x00);
            assert_eq!(mc, 0x00);

            let mc: u8 = create_mc(RW::ReadAccess, CommunicationChannel::Page, 0x02);
            assert_eq!(mc, 0xA2);
        }
    }
}
