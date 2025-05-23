#![cfg_attr(not(test), no_std)]

use embedded_hal_async::i2c::I2c;
use embedded_hal::digital::OutputPin;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Led {
    LED1,
    LED2,
}

#[allow(non_camel_case_types)]
pub struct Pins<T: OutputPin> {
    #[allow(non_snake_case)]
    pub ENL_plus: T,
}

#[derive(Debug)]
pub enum Error {
    Invalid7bitAddress,
}

pub struct L6360<I2cType: I2c, OutputPinType: OutputPin> {
    i2c: I2cType,
    address_7bit: u8,
    pins: Pins<OutputPinType>
}

impl<I2cType: I2c, OutputPinType: OutputPin> L6360<I2cType, OutputPinType>
{
    pub fn new(i2c: I2cType, address_7bit: u8, pins: Pins<OutputPinType>) -> Result<Self, Error>
    {
        // address is 7 bit and must have the form 0b1100_xxx
        if !(0b0_1100_000..=0b0_1100_111).contains(&address_7bit) {
            return Err(Error::Invalid7bitAddress);
        }

        Ok(Self {
            i2c,
            address_7bit,
            pins,
        })
    }

    #[allow(non_snake_case)]
    pub fn enable_ENL_plus(&mut self) -> Result<(), OutputPinType::Error> {
        self.pins.ENL_plus.set_high()
    }

    pub async fn set_control_register_1(&mut self) -> Result<(), I2cType::Error> {
        let data: u8 = 0b1010_0001;
        let parity = Self::calculate_parity(data);
        let parity_and_reg_addr = (parity << 5) | (0b0010 as u8);
        self.i2c.write(self.address_7bit, &[data, parity_and_reg_addr]).await?;
        Ok(())
    }

    pub async fn set_led_pattern(&mut self, led: Led, pattern: u16) -> Result<(), I2cType::Error> {
        let led_pattern_msb_lsb = [(pattern >> 8) as u8, pattern as u8];

        let reg_addr_start = match led {
            Led::LED1 => 0b0100,
            Led::LED2 => 0b0110,
        };

        for i in 0..2 {
            let parity = Self::calculate_parity(led_pattern_msb_lsb[i]);
            let parity_and_reg_addr = (parity << 5) | (reg_addr_start + i as u8);
            self.i2c.write(self.address_7bit, &[led_pattern_msb_lsb[i], parity_and_reg_addr]).await?;
        }
        Ok(())
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
    use super::{L6360, Led};
    use embedded_hal_async::i2c;
    use tokio;
    use mockall::*;

    mock! {
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

    #[tokio::test]
    async fn test_new() {
        for address in 0..=255 {
            let i2c_mock = MockI2c::new();
            let result = L6360::new(i2c_mock, address);
            if address < 0b0_1100_000 || address > 0b0_1100_111 {
                assert!(result.is_err(), "L6360::new returned ok, with address: {:?}", address);
            }
            else {
                assert!(result.is_ok(), "L6360::new returned err: {:?}, with address: {:?}", result.err().unwrap(), address);
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

            let mut i2c_mock = MockI2c::new();

            i2c_mock
                .expect_write()
                .times(1)
                .withf(move |address, bytes| {
                    *address == *i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *write_byte_00 &&
                    bytes[1] == *write_byte_10
                })
                .returning(|_, _| Ok(()));

            i2c_mock
                .expect_write()
                .times(1)
                .withf(move |address, bytes| {
                    *address == *i2c_address &&
                    bytes.len() == 2 &&
                    bytes[0] == *write_byte_01 &&
                    bytes[1] == *write_byte_11
                })
                .returning(|_, _| Ok(()));

            let mut l63601 = L6360::new(i2c_mock, *i2c_address).unwrap();
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
            assert_eq!(L6360::<MockI2c>::calculate_parity(*data), *expected);
        }
    }
}

