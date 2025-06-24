
//! Driver for the ST L6360 IO-Link communication master transceiver IC

#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use mockall::automock;

//#[cfg(feature = "log")]
//use log::info;
//#[cfg(feature = "defmt")]
//use defmt::info;

use embedded_hal_async::i2c::{self, I2c};
use num_enum::TryFromPrimitive;
pub use embedded_hal::digital::PinState;

/// Implementations for hardware access.
#[cfg_attr(test, automock)]
pub trait HardwareAccess {
    fn enl_plus(&mut self, level: PinState);
    fn en_cq(&mut self, level: PinState);
    fn in_cq(&mut self, level: PinState);
    fn out_cq(&self) -> PinState;
    #[allow(async_fn_in_trait)]
    async fn exchange(&mut self, data: &[u8], answer: &mut [u8]);
}

enum RegisterAddress {
    // Status        = 0b0000,
    Configuration = 0b0001,
    // Control1      = 0b0010,
    // Control2      = 0b0011,
    // #[allow(non_camel_case_types)]
    // LED1_MSB      = 0b0100,
    // #[allow(non_camel_case_types)]
    // LED1_LSB      = 0b0101,
    // #[allow(non_camel_case_types)]
    // LED2_MSB      = 0b0110,
    // #[allow(non_camel_case_types)]
    // LED2_LSB      = 0b0111,
    // Parity        = 0b1000,
}

/// Configurations for the C/Q output stage.
#[derive(Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum CqOutputStageConfiguration {
    OFF        = 0b000,
    LowSide    = 0b001,
    HighSide   = 0b010,
    PushPull   = 0b011,
    TriState   = 0b100,
    LowSideON  = 0b101,
    HighSideON = 0b110,
}

pub struct ConfigurationRegister {
    pub cq_output_stage_configuration: CqOutputStageConfiguration,
}

/// Values for EN_CGQ of [`ControlRegister1`]
#[derive(PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum EN_CGQ_CQ_PullDown {
    /// Always OFF
    OFF,
    /// ON if EN_C/Q = 0 and OFF if EN_C/Q = 1
    ON_IfEnCq0,
}

/// Configuration of Control register 1. See [`Config`]
pub struct ControlRegister1 {
    pub en_cgq_cq_pull_down: EN_CGQ_CQ_PullDown,
}

/// Configuration
pub struct Config {
    pub configuration_register: ConfigurationRegister,
    pub control_register_1: ControlRegister1,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            configuration_register: ConfigurationRegister {
                cq_output_stage_configuration: CqOutputStageConfiguration::TriState,
            },
            control_register_1: ControlRegister1 {
                en_cgq_cq_pull_down: EN_CGQ_CQ_PullDown::OFF,
            },
        }
    }
}

/// LEDs
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Led {
    LED1,
    LED2,
}

/// Error
#[derive(Debug)]
pub enum Error<I2cError> {
    Invalid7bitAddress,
    InvalidRegisterAddress,
    I2cError(I2cError),
}

type L6360result<T, I2C> = Result<T, Error<<I2C as i2c::ErrorType>::Error>>;

/// Struct for the L6360
/// Use [`L6360::new`] to create an instance of this struct.
pub struct L6360<I2C, HW> {
    i2c: I2C,
    /// Hardware can be accessed via this field containig the concrete implementations.
    pub hw: HW,
    address_7bit: i2c::SevenBitAddress,
    config: Config,
}

impl<I2C, HW> L6360<I2C, HW>
where
    I2C: I2c,
    HW: HardwareAccess,
{
    /// Creates a new instance of the L6360 driver.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C interface to communicate with the L6360.
    /// * `hw` - The hardware access implementation to control the L6360.
    /// * `address_7bit` - The 7-bit I2C address of the L6360. Via hardware pins the L6360 address is settable from `0b0_1100_000` to `0b0_1100_111`.
    /// * `config` - The configuration for the L6360.
    ///
    /// # Returns
    ///
    /// A result containing the L6360 instance or an error.
    pub fn new(i2c: I2C, hw: HW, address_7bit: i2c::SevenBitAddress, config: Config) -> L6360result<Self, I2C> {
        if !(0b0_1100_000..=0b0_1100_111).contains(&address_7bit) {
            return Err(Error::Invalid7bitAddress);
        }

        Ok(Self {
            i2c,
            hw,
            address_7bit,
            config,
        })
    }

    /// Initializes the L6360.
    pub async fn init(&mut self) -> L6360result<(), I2C> {
        self.set_cq_out_stage_configuration(self.config.configuration_register.cq_output_stage_configuration).await?;
        self.init_control_register_1().await?;
        Ok(())
    }

    async fn init_control_register_1(&mut self) -> L6360result<(), I2C> {
        let data: u8 = if self.config.control_register_1.en_cgq_cq_pull_down == EN_CGQ_CQ_PullDown::ON_IfEnCq0 {
            0b1010_0001
        }
        else {
            0b0010_0001
        };
        self.write_register(0b0010, data).await?;
        Ok(())
    }

    /// Sets the C/Q output stage configuration.
    async fn set_cq_out_stage_configuration(&mut self, config: CqOutputStageConfiguration) -> L6360result<(), I2C> {
        const CONF_REG_ADDR: u8 = RegisterAddress::Configuration as u8;
        const BIT_SHIFT: u8 = 5;
        let current_register_value = self.read_register_random(CONF_REG_ADDR).await.unwrap() >> BIT_SHIFT;
        match CqOutputStageConfiguration::try_from(current_register_value).unwrap() {
            CqOutputStageConfiguration::OFF |
            CqOutputStageConfiguration::TriState => (),

            CqOutputStageConfiguration::LowSide |
            CqOutputStageConfiguration::HighSide |
            CqOutputStageConfiguration::PushPull |
            CqOutputStageConfiguration::LowSideON |
            CqOutputStageConfiguration::HighSideON => {
                self.write_register(CONF_REG_ADDR, (CqOutputStageConfiguration::OFF as u8) << BIT_SHIFT).await?;
            }
        }

        let register_value = (config as u8) << BIT_SHIFT;
        self.write_register(CONF_REG_ADDR, register_value).await?;
        Ok(())
    }

    /// Sets a pattern for the specified Led.
    ///
    /// # Arguments
    ///
    /// * `led` - The [`Led`] to set the pattern for.
    /// * `pattern` - A 16-bit pattern to set for the Led.
    ///   The 16 bits are applied round robin with a rate of 63ms (16 * 63ms = 1008ms).
    ///   The Led is on during 1 bits and off during 0 bits.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn set_led_pattern(&mut self, led: Led, pattern: u16) -> L6360result<(), I2C>{
        let led_pattern_msb_lsb = [(pattern >> 8) as u8, pattern as u8];

        let reg_addr_start = match led {
            Led::LED1 => 0b0100,
            Led::LED2 => 0b0110,
        };

        for i in 0..=1 {
            self.write_register(reg_addr_start + i as u8, led_pattern_msb_lsb[i]).await?;
        }
        Ok(())
    }

    async fn write_register(&mut self, register_address: u8, data: u8) -> L6360result<(), I2C> {
        if !(0b0000..=0b1000).contains(&register_address) {
            return Err(Error::InvalidRegisterAddress);
        }
        let parity = Self::calculate_parity(data);
        let parity_and_reg_addr = (parity << 5) | (register_address);
        self.i2c.write(self.address_7bit, &[data, parity_and_reg_addr]).await.map_err(Error::I2cError)?;
        Ok(())
    }

    async fn read_register_random(&mut self, register_address: u8)  -> L6360result<u8, I2C> {
        if !(0b0000..=0b1000).contains(&register_address) {
            return Err(Error::InvalidRegisterAddress);
        }

        self.i2c.write(self.address_7bit, &[register_address]).await.map_err(Error::I2cError)?;
        let mut buf = [0u8; 1];
        self.i2c.read(self.address_7bit, &mut buf).await.map_err(Error::I2cError)?;
        Ok(buf[0])

    }

    fn calculate_parity(data: u8) -> u8 {
        let d0 = (data >> 0) & 1;
        let d1 = (data >> 1) & 1;
        let d2 = (data >> 2) & 1;
        let d3 = (data >> 3) & 1;
        let d4 = (data >> 4) & 1;
        let d5 = (data >> 5) & 1;
        let d6 = (data >> 6) & 1;
        let d7 = (data >> 7) & 1;

        let p0 = d7 ^ d6 ^ d5 ^ d4 ^ d3 ^ d2 ^ d1 ^ d0;
        let p1 = d7 ^ d5 ^ d3 ^ d1;
        let p2 = d6 ^ d4 ^ d2 ^ d0;

        (p2 << 2) | (p1 << 1) | p0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use mockall::*;

    mock! {
        #[derive(Debug)]
        pub I2c {}

        impl i2c::ErrorType for I2c {
            type Error = core::convert::Infallible;
        }

        impl i2c::I2c for I2c {
            async fn read(&mut self, address: i2c::SevenBitAddress, buffer: &mut [u8]) -> Result<(), <Self as i2c::ErrorType>::Error>;
            async fn write(&mut self, address: i2c::SevenBitAddress, bytes: &[u8]) -> Result<(), <Self as i2c::ErrorType>::Error>;
            // async fn write_read(&mut self, address: i2c::SevenBitAddress, bytes: &[u8], buffer: &mut [u8]) -> Result<(), <Self as i2c::ErrorType>::Error>;
            async fn transaction<'a>(&mut self, address: i2c::SevenBitAddress, operations: &mut [i2c::Operation<'a>]) -> Result<(), <Self as i2c::ErrorType>::Error>;
        }
    }

    #[test]
    fn test_new() {
        for address in 0..=255 {
            let mock_i2c = MockI2c::new();
            let mock_hw = MockHardwareAccess::new();
            let config = Config::default();
            let result = L6360::new(mock_i2c, mock_hw, address, config);
            if address < 0b0_1100_000 || address > 0b0_1100_111 {
                assert!(result.is_err(), "L6360::new returned ok, with address: {:?}", address);
            }
            else {
                assert!(result.is_ok(), "L6360::new returned err: {:?}, with address: {:?}", result.err().unwrap(), address);
            }
        }
    }

    #[tokio::test]
    async fn test_init() {
        let expected = &[
            //setting                        //expectation
            (EN_CGQ_CQ_PullDown::OFF,        0b0010_0001),
            (EN_CGQ_CQ_PullDown::ON_IfEnCq0, 0b1010_0001),
        ];

        for (en_cgq_cq_pulldown, reg_value) in expected.iter() {
            let mut mock_i2c = MockI2c::new();
            let mock_hw = MockHardwareAccess::new();
            let i2c_address = 0b0_1100_111;
            let config = Config {
                control_register_1: ControlRegister1 {
                    en_cgq_cq_pull_down: *en_cgq_cq_pulldown,
                },
                ..Default::default()
            };

            mock_i2c.expect_read().times(1)
                .returning(|_, _| Ok(()));
            mock_i2c.expect_write().times(2)
                .returning(|_, _| Ok(()));

            mock_i2c.expect_write().times(1)
                .withf(move |address, bytes| {
                    *address == i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *reg_value &&
                    bytes[1] & 0b0000_1111 == 0b0010 // We only care for the register address here.
                })
                .returning(|_, _| Ok(()));

            let mut l6360 = L6360::new(mock_i2c, mock_hw, i2c_address, config).unwrap();
            l6360.init().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_set_cq_out_stage_configuration() {
        for current_config in [
            CqOutputStageConfiguration::OFF,
            CqOutputStageConfiguration::LowSide,
            CqOutputStageConfiguration::HighSide,
            CqOutputStageConfiguration::PushPull,
            CqOutputStageConfiguration::TriState,
            CqOutputStageConfiguration::LowSideON,
            CqOutputStageConfiguration::HighSideON,
        ]
        {
            for new_config in [
                CqOutputStageConfiguration::OFF,
                CqOutputStageConfiguration::LowSide,
                CqOutputStageConfiguration::HighSide,
                CqOutputStageConfiguration::PushPull,
                CqOutputStageConfiguration::TriState,
                CqOutputStageConfiguration::LowSideON,
                CqOutputStageConfiguration::HighSideON,
            ]
            {
                let mut mock_i2c = MockI2c::new();
                let mock_hw = MockHardwareAccess::new();
                let i2c_address = 0b0_1100_111;

                // expect set of start register address
                mock_i2c.expect_write().times(1)
                    .withf(move |address, bytes| {
                        *address == i2c_address &&
                        bytes.len() == 1 &&
                        bytes[0] == 0b0001
                    })
                    .returning(|_, _| Ok(()));

                // expect reading of current config
                mock_i2c.expect_read().times(1)
                    .withf(move |address, _| {
                        *address == i2c_address
                    })
                    .returning(move |_, bytes| {
                        bytes.copy_from_slice(&[(current_config as u8) << 5]);
                        Ok(())
                    });

                match current_config {
                    // Note: List all values here so we can be sure that all values are tested.
                    CqOutputStageConfiguration::OFF |
                    CqOutputStageConfiguration::TriState => (),
                    // On some values we need to transit via OFF config.
                    CqOutputStageConfiguration::LowSide |
                    CqOutputStageConfiguration::HighSide |
                    CqOutputStageConfiguration::PushPull |
                    CqOutputStageConfiguration::LowSideON |
                    CqOutputStageConfiguration::HighSideON => {
                        mock_i2c
                            .expect_write()
                            .times(1)
                            .withf(move |address, bytes| {
                                *address == i2c_address &&
                                bytes.len() == 2 &&
                                bytes[0] == (CqOutputStageConfiguration::OFF as u8) << 5
                            })
                            .returning(|_, _| Ok(()));
                    }
                }

                // expect writing to the final config
                mock_i2c.expect_write().times(1)
                    .withf(move |address, bytes| {
                        *address == i2c_address &&
                        bytes.len() == 2 &&
                        bytes[0] == (new_config as u8) << 5
                    })
                    .returning(|_, _| Ok(()));

                let mut l6360 = L6360::new(mock_i2c, mock_hw, i2c_address, Config::default()).unwrap();
                l6360.set_cq_out_stage_configuration(new_config).await.unwrap();
            }
        }
    }

    #[tokio::test]
    async fn test_set_led_pattern() {
        let test_cases: &[(
               // input                          // expect
               u8,           Led,       u16,     u8,            u8,            u8,            u8 )] = &[
            // i2c_address,  led,       pattern, write_byte_00, write_byte_10, write_byte_01, write_byte_11
            (  0b0_1100_101, Led::LED1, 0x0000,  0x00,          0x04,          0x00,          0x05, ),
            (  0b0_1100_111, Led::LED2, 0x0000,  0x00,          0x06,          0x00,          0x07, ),
            (  0b0_1100_000, Led::LED1, 0x55AA,  0x55,          0x04,          0xAA,          0x05, ),
            (  0b0_1100_001, Led::LED2, 0x1234,  0x12,          0xC6,          0x34,          0x67, ),

        ];

        let mut test_cnt = 0;
        for (i2c_address, led, pattern, write_byte_00, write_byte_10, write_byte_01, write_byte_11) in test_cases {
            println!("test_cnt: {:?}", test_cnt);
            test_cnt += 1;

            let mut mock_i2c = MockI2c::new();
            let mock_hw = MockHardwareAccess::new();

            mock_i2c.expect_write().times(1)
                .withf(move |address, bytes| {
                    *address == *i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *write_byte_00 &&
                    bytes[1] == *write_byte_10
                })
                .returning(|_, _| Ok(()));

            mock_i2c.expect_write().times(1)
                .withf(move |address, bytes| {
                    *address == *i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *write_byte_01 &&
                    bytes[1] == *write_byte_11
                })
                .returning(|_, _| Ok(()));

            let mut l63601 = L6360::new(mock_i2c, mock_hw, *i2c_address, Config::default()).unwrap();
            l63601.set_led_pattern(*led, *pattern).await.unwrap();
        }
    }

    #[test]
    fn test_calculate_parity() {
        let test_cases: &[(u8, u8)] = &[
            // all on or off
            (0b0000_0000, 0b000),
            (0b1111_1111, 0b000),
            // alternating
            (0b1010_1010, 0b000),
            (0b0101_0101, 0b000),
            // one bit
            (0b0000_0001, 0b101),
            (0b0000_0010, 0b011),
            (0b0000_0100, 0b101),
            (0b0000_1000, 0b011),
            (0b0001_0000, 0b101),
            (0b0010_0000, 0b011),
            (0b0100_0000, 0b101),
            (0b1000_0000, 0b011),
            // random
            (0b1001_0101, 0b110),
            (0b0011_0010, 0b101),
        ];

        println!("|    data    | expected |");
        println!("|:----------:|:--------:|");
        for (data, expected) in test_cases {
            println!("| 0b{:08b} |   0b{:03b}  |", data, expected);
            assert_eq!(L6360::<MockI2c, MockHardwareAccess>::calculate_parity(*data), *expected);
        }
    }
}
