// see #B.1.2

pub mod direct_parameter_page_1_and_2 {
    pub mod address {
        // #[allow(non_upper_case_globals)]
        // pub const MasterCommand: u8 = 0x00;
        // #[allow(non_upper_case_globals)]
        // pub const MasterCycleTime: u8 = 0x01;
        #[allow(non_upper_case_globals)]
        pub const MinCycleTime: u8 = 0x02;
    }
}
