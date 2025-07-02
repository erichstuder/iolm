//! Annex A
//!
//! see [#Annex A](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=230)

// This a workaround as this code in here is shown unused although used.
// The compiler shows no warnings. Looks like the rust-analyzer has a problem here.
#![allow(dead_code)]

pub enum RW {
    #[allow(unused)] // TODO: remove
    WriteAccess = 0,
    ReadAccess = 1,
}

pub enum CommunicationChannel {
    #[allow(unused)] // TODO: remove
    Process = 0,
    Page = 1,
    // Diagnosis = 2,
    // ISDU = 3,
}


#[allow(non_camel_case_types)]
pub enum M_Sequence_Type {
    #[allow(non_camel_case_types)]
    Type_0 = 0,
    // #[allow(non_camel_case_types)]
    // Type_1 = 1,
    // #[allow(non_camel_case_types)]
    // Type_2 = 2,
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

pub fn calculate_checksum(message: &mut [u8]) {
    const SEED: u8 = 0x52;
    let mut result = SEED;
    for m in message.iter() {
        result ^= *m;
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
    fn test_create_mc() {
        let mc: u8 = create_mc(RW::WriteAccess, CommunicationChannel::Process, 0x00);
        assert_eq!(mc, 0x00);

        let mc: u8 = create_mc(RW::ReadAccess, CommunicationChannel::Page, 0x02);
        assert_eq!(mc, 0xA2);
    }

    #[test]
    #[should_panic(expected = "invalid address size")]
    fn test_create_mc_panic() {
        let mc: u8 = create_mc(RW::ReadAccess, CommunicationChannel::Page, 0b1_1111+1);
    }

    #[test]
    fn test_create_ckt_without_checksum() {
        let ckt = create_ckt_without_checksum(M_Sequence_Type::Type_0);
        assert_eq!(ckt, 0x00);
    }

    #[test]
    fn test_calculate_checksum() {
        let mut message: [u8; 4] = [1,2,3,4];
        let checksum = calculate_checksum(&mut message);
        assert_eq!(message[0], 1);
        assert_eq!(message[1], 63);
        assert_eq!(message[2], 3);
        assert_eq!(message[3], 4);
    }
}
