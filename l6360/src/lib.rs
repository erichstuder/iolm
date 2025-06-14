#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use mockall::automock;

//#[cfg(feature = "log")]
//use log::info;
//#[cfg(feature = "defmt")]
//use defmt::info;

use embedded_hal_async::i2c::{self, I2c};
use embedded_hal::digital::OutputPin;
use num_enum::TryFromPrimitive;
pub use embedded_hal::digital::PinState;

#[cfg_attr(test, automock)]
pub trait Uart {
    fn in_cq(&mut self, level: PinState);
    fn out_cq(&self) -> PinState;
    fn exchange(&mut self, data: &[u8], answer: &[u8]) -> Result<usize, ()>;
}

#[derive(PartialEq, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum EN_CGQ_CQ_PullDown {
    OFF,
    ON_IfEnCq0,
}

pub struct ControlRegister1 {
    pub en_cgq_cq_pulldown: EN_CGQ_CQ_PullDown,
}

#[derive(TryFromPrimitive)]
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

pub struct Config {
    pub control_register_1: ControlRegister1,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            control_register_1: ControlRegister1 {
                en_cgq_cq_pulldown: EN_CGQ_CQ_PullDown::OFF
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Led {
    LED1,
    LED2,
}

pub struct Pins<OutputPinType>
where
    OutputPinType: OutputPin,
{
    pub enl_plus: OutputPinType,
    pub en_cq: OutputPinType,
}

#[derive(Debug)]
pub enum Error<I2cError> {
    Invalid7bitAddress,
    InvalidRegisterAddress,
    I2cError(I2cError),
}

type L6360result<T, I2C> = Result<T, Error<<I2C as i2c::ErrorType>::Error>>;

pub struct L6360<I2C, U, OutputPinType>
where
    I2C: I2c,
    U: Uart,
    OutputPinType: OutputPin,
{
    i2c: I2C,
    pub uart: U,
    address_7bit: i2c::SevenBitAddress,
    pub pins: Pins<OutputPinType>,
    config: Config,
}

impl<I2C, U, OutputPinType> L6360<I2C, U, OutputPinType>
where
    I2C: I2c,
    U: Uart,
    OutputPinType: OutputPin,
{
    pub fn new(i2c: I2C, uart: U, address_7bit: i2c::SevenBitAddress, pins: Pins<OutputPinType>, config: Config) -> L6360result<Self, I2C> {
        if !(0b0_1100_000..=0b0_1100_111).contains(&address_7bit) {
            return Err(Error::Invalid7bitAddress);
        }

        Ok(Self {
            i2c,
            uart,
            address_7bit,
            pins,
            config,
        })
    }

    pub async fn init(&mut self) -> L6360result<(), I2C> {
        self.init_control_register_1().await?;
        Ok(())
    }

    async fn init_control_register_1(&mut self) -> L6360result<(), I2C> {
        let data: u8 = if self.config.control_register_1.en_cgq_cq_pulldown == EN_CGQ_CQ_PullDown::ON_IfEnCq0 {
            0b1010_0001
        }
        else {
            0b0010_0001
        };
        self.write_register(0b0010, data).await?;
        Ok(())
    }

    pub async fn set_cq_out_stage_configuration(&mut self, config: CqOutputStageConfiguration) -> L6360result<(), I2C> {
        const CONFIG_REGISTER_ADDRESS: u8 = 0b0001;
        const BIT_SHIFT: u8 = 5;
        let current_register_value = self.read_register_random(CONFIG_REGISTER_ADDRESS).await.unwrap() >> BIT_SHIFT;
        match CqOutputStageConfiguration::try_from(current_register_value).unwrap() {
            CqOutputStageConfiguration::OFF |
            CqOutputStageConfiguration::TriState => (),

            CqOutputStageConfiguration::LowSide |
            CqOutputStageConfiguration::HighSide |
            CqOutputStageConfiguration::PushPull |
            CqOutputStageConfiguration::LowSideON |
            CqOutputStageConfiguration::HighSideON => {
                self.write_register(CONFIG_REGISTER_ADDRESS, CqOutputStageConfiguration::OFF as u8).await?;
            }
        }

        let register_value = (config as u8) << BIT_SHIFT;
        self.write_register(CONFIG_REGISTER_ADDRESS, register_value).await?;
        Ok(())
    }

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
    use embedded_hal::digital;
    use tokio;
    use mockall::*;

    mock! {
        #[derive(Debug)]
        pub I2c {}

        impl i2c::ErrorType for I2c {
            type Error = core::convert::Infallible;
        }

        impl i2c::I2c for I2c {
            // async fn read(&mut self, address: i2c::SevenBitAddress, buffer: &mut [u8]) -> Result<(), <Self as i2c::ErrorType>::Error>;
            async fn write(&mut self, address: i2c::SevenBitAddress, bytes: &[u8]) -> Result<(), <Self as i2c::ErrorType>::Error>;
            // async fn write_read(&mut self, address: i2c::SevenBitAddress, bytes: &[u8], buffer: &mut [u8]) -> Result<(), <Self as i2c::ErrorType>::Error>;
            async fn transaction<'a>(&mut self, address: i2c::SevenBitAddress, operations: &mut [i2c::Operation<'a>]) -> Result<(), <Self as i2c::ErrorType>::Error>;
        }
    }

    mock! {
        pub OutputPinType {}

        impl digital::ErrorType for OutputPinType {
            type Error = core::convert::Infallible;
        }

        impl OutputPin for OutputPinType {
            fn set_low(&mut self) -> Result<(), <Self as digital::ErrorType>::Error>;
            fn set_high(&mut self) -> Result<(), <Self as digital::ErrorType>::Error>;
            fn set_state(&mut self, state: PinState) -> Result<(), <Self as digital::ErrorType>::Error>;
        }
    }

    // mock! {
    //     pub InputPinType {}

    //     impl digital::ErrorType for InputPinType {
    //         type Error = core::convert::Infallible;
    //     }

    //     impl InputPin for InputPinType {
    //         fn is_high(&mut self) -> Result<bool, <Self as digital::ErrorType>::Error>;
    //         fn is_low(&mut self) -> Result<bool, <Self as digital::ErrorType>::Error>;
    //     }
    // }

    #[tokio::test]
    async fn test_new() {
        for address in 0..=255 {
            let mock_i2c = MockI2c::new();
            let mock_uart = MockUart::new();
            let pins = Pins {
                enl_plus: MockOutputPinType::new(),
                en_cq: MockOutputPinType::new(),
            };
            let config = Config::default();
            let result = L6360::new(mock_i2c, mock_uart, address, pins, config);
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
        let en_cgq_cq_pulldown = &[
            //setting                        //expectation
            (EN_CGQ_CQ_PullDown::OFF,        0b0010_0001),
            (EN_CGQ_CQ_PullDown::ON_IfEnCq0, 0b1010_0001),
        ];

        for (en_cgq_cq_pulldown, reg_value) in en_cgq_cq_pulldown.iter() {
            let mut mock_i2c = MockI2c::new();
            let mock_uart = MockUart::new();
            let i2c_address = 0b0_1100_111;
            let pins = Pins {
                enl_plus: MockOutputPinType::new(),
                en_cq: MockOutputPinType::new(),
            };
            let config = Config {
                control_register_1: ControlRegister1 {
                    en_cgq_cq_pulldown: *en_cgq_cq_pulldown
                }
            };

            mock_i2c
                .expect_write()
                .times(1)
                .withf(move |address, bytes| {
                    *address == i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *reg_value &&
                    bytes[1] & 0b0000_1111 == 0b0010 // We only care for the register address here.
                })
                .returning(|_, _| Ok(()));

            let mut l6360 = L6360::new(mock_i2c, mock_uart, i2c_address, pins, config).unwrap();
            l6360.init().await.unwrap();
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
            let mock_uart = MockUart::new();
            let pins = Pins {
                enl_plus: MockOutputPinType::new(),
                en_cq: MockOutputPinType::new(),
            };

            mock_i2c
                .expect_write()
                .times(1)
                .withf(move |address, bytes| {
                    *address == *i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *write_byte_00 &&
                    bytes[1] == *write_byte_10
                })
                .returning(|_, _| Ok(()));

            mock_i2c
                .expect_write()
                .times(1)
                .withf(move |address, bytes| {
                    *address == *i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *write_byte_01 &&
                    bytes[1] == *write_byte_11
                })
                .returning(|_, _| Ok(()));

            let mut l63601 = L6360::new(mock_i2c, mock_uart, *i2c_address, pins, Config::default()).unwrap();
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
            assert_eq!(L6360::<MockI2c, MockUart, MockOutputPinType>::calculate_parity(*data), *expected);
        }
    }
}
