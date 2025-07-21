use defmt::info;
use embassy_stm32::bind_interrupts;
use embassy_stm32::i2c::I2c;
use embassy_stm32::usart::{self, Uart};
use embassy_stm32::peripherals;
use embassy_stm32::gpio::{Output, Input, Level, Speed, Pull};
use embassy_stm32::Peripheral;
use embassy_stm32::mode::Async;
use embassy_time::{Duration, with_timeout};

use l6360::{L6360, Led};
pub use l6360::PinState;

// bind_interrupts!(struct I2cIrqs {
//     I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
//     I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
// });

bind_interrupts!(struct UartIrqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Gpio,
    Uart,
}

// Note: bind_interrupts is not generic. This leads to having concrete types.
#[allow(non_camel_case_types)]
pub struct IOL_Transceiver<'a> {
    l6360: L6360<I2c<'a, Async>>,
    uart_instance: Option<peripherals::USART1>,
    tx_dma: Option<peripherals::DMA2_CH7>,
    rx_dma: Option<peripherals::DMA2_CH2>,
    tx_pin: Option<peripherals::PA9>,
    rx_pin: Option<peripherals::PA10>,
    enl_plus: Output<'a>,
    en_cq: Output<'a>,
    in_cq: Option<Output<'a>>,
    out_cq: Option<Input<'a>>,
    uart: Option<Uart<'a, Async>>,
    mode: Mode,
}

impl<'a> IOL_Transceiver<'a> {
    pub async fn new(
        i2c: I2c<'a, Async>,
        uart_instance: peripherals::USART1,
        tx_dma: peripherals::DMA2_CH7,
        rx_dma: peripherals::DMA2_CH2,
        tx_pin: peripherals::PA9,
        rx_pin: peripherals::PA10,
        enl_plus: peripherals::PA6,
        en_cq: peripherals::PC0,
    ) -> Self {
        // Note: This struct uses unsafe to be able to switch between gpio and uart.
        #[allow(unsafe_code)]
        let tx_clone = unsafe { tx_pin.clone_unchecked() };
        #[allow(unsafe_code)]
        let rx_clone = unsafe { rx_pin.clone_unchecked() };

        // let l6360_hw = IOL_Transceiver::new(p.USART1, p.DMA2_CH7, p.DMA2_CH2, p.PA9, p.PA10, p.PA6, p.PC0);

        let config = l6360::Config {
            configuration_register: l6360::ConfigurationRegister {
                cq_output_stage_configuration: l6360::CqOutputStageConfiguration::PushPull,
            },
            control_register_1: l6360::ControlRegister1 {
                en_cgq_cq_pull_down: l6360::EN_CGQ_CQ_PullDown::ON_IfEnCq0,
            }
        };

        let mut l6360 = L6360::new(i2c, 0b1100_000, config).unwrap();
        l6360.init().await.unwrap();

        Self {
            l6360,
            uart_instance: Some(uart_instance),
            tx_dma: Some(tx_dma),
            rx_dma: Some(rx_dma),
            tx_pin: Some(tx_pin),
            rx_pin: Some(rx_pin),
            enl_plus: Output::new(enl_plus, Level::Low, Speed::Low),
            en_cq: Output::new(en_cq, Level::Low, Speed::Low),
            in_cq: Some(Output::new(tx_clone, Level::Low, Speed::Low)),
            out_cq: Some(Input::new(rx_clone, Pull::None)),
            uart: None,
            mode: Mode::Gpio,
        }
    }

    pub async fn set_led_pattern(&mut self, led: Led, pattern: u16) {
        self.l6360.set_led_pattern(led, pattern).await.unwrap();
    }

    pub fn switch_to_uart(&mut self) {
        // Drop gpios before initalizing uart.
        drop(self.in_cq.take());
        drop(self.out_cq.take());

        let mut config = usart::Config::default();
        config.baudrate = 9600; // just a random value
        config.data_bits = usart::DataBits::DataBits8;
        config.stop_bits = usart::StopBits::STOP1;
        config.parity = usart::Parity::ParityEven;
        config.detect_previous_overrun = true;
        config.assume_noise_free = false;
        config.rx_pull = Pull::None;

        self.uart = Some(Uart::new(
            self.uart_instance.take().unwrap(),
            self.rx_pin.take().unwrap(),
            self.tx_pin.take().unwrap(),
            UartIrqs,
            self.tx_dma.take().unwrap(),
            self.rx_dma.take().unwrap(),
            config,
        ).unwrap());

        self.mode = Mode::Uart;
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }

    pub fn enl_plus(&mut self, level: l6360::PinState) {
        match level {
            l6360::PinState::High => self.enl_plus.set_level(Level::High),
            l6360::PinState::Low => self.enl_plus.set_level(Level::Low),
        }
    }

    pub fn en_cq(&mut self, level: l6360::PinState) {
        match level {
            l6360::PinState::High => self.en_cq.set_level(Level::High),
            l6360::PinState::Low => self.en_cq.set_level(Level::Low),
        }
    }

    pub fn in_cq(&mut self, level: l6360::PinState) {
        let pin = self.in_cq.as_mut().unwrap();
        match level {
            l6360::PinState::High => pin.set_level(Level::High),
            l6360::PinState::Low => pin.set_level(Level::Low),
        }
    }

    pub fn out_cq(&self) -> l6360::PinState {
        match self.out_cq.as_ref().unwrap().get_level() {
            Level::High => l6360::PinState::High,
            Level::Low => l6360::PinState::Low,
        }
    }

    pub fn set_baudrate(&mut self, baudrate: u32) {
        self.uart.as_ref().unwrap().set_baudrate(baudrate).unwrap();
    }


    fn convert_uart_error(err: usart::Error) -> iol::master::TransferError {
        match err {
            usart::Error::Framing => iol::master::TransferError::FRAMING_ERROR,
            usart::Error::Noise => panic!("noise error"), // No corresponding error. So just panic.
            usart::Error::Overrun => iol::master::TransferError::OVERRUN,
            usart::Error::Parity => iol::master::TransferError::PARITY_ERROR,
            usart::Error::BufferTooLong => panic!("buffer too long error"), // Programming error. So just panic.
            e => panic!("unhandled error: {:?}", e),
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), iol::master::TransferError>{
        self.en_cq(l6360::PinState::High);
        self.uart.as_mut().unwrap().blocking_write(data).map_err(Self::convert_uart_error)?;
        self.uart.as_mut().unwrap().blocking_flush().map_err(Self::convert_uart_error)?;
        self.en_cq(l6360::PinState::Low); // TODO: this should not be necessary
        Ok(())
    }

    pub async fn try_receive(&mut self, answer: &mut Option<&mut [u8]>) -> Result<(), iol::master::TransferError>{
        self.en_cq(l6360::PinState::Low);
        if let Some(buffer) = answer {
            match embassy_time::with_timeout(
                Duration::from_millis(1),
                self.uart.as_mut().unwrap().read(buffer)
            ).await {
                Ok(Ok(())) => {
                    info!("ok");
                    for byte in buffer.iter(){
                        info!("answer: {:#04x}", byte);
                    }
                    Ok(())
                }
                Ok(Err(e)) => {
                    info!("uart error: {:?}", e);
                    Err(Self::convert_uart_error(e))
                }
                Err(_) => {
                    info!("No answer for now");
                    *answer = None;
                    Ok(())
                }
            }
        } else {
            panic!("must be Some");
        }
    }
}
