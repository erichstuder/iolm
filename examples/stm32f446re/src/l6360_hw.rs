use defmt::info;
use embassy_stm32::bind_interrupts;
use embassy_stm32::usart::{self, Uart};
use embassy_stm32::peripherals;
use embassy_stm32::gpio::{Output, Input, Level, Speed, Pull};
use embassy_stm32::Peripheral;
use embassy_stm32::mode::Blocking;

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
pub struct L6360_HW<'a> {
    uart_instance: Option<peripherals::USART1>,
    tx_pin: Option<peripherals::PA9>,
    rx_pin: Option<peripherals::PA10>,
    enl_plus: Output<'a>,
    en_cq: Output<'a>,
    in_cq: Option<Output<'a>>,
    out_cq: Option<Input<'a>>,
    uart: Option<Uart<'a, Blocking>>,
    mode: Mode,
}

impl<'a> L6360_HW<'a> {
    pub fn new(
        uart_instance: peripherals::USART1,
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

        Self {
            uart_instance: Some(uart_instance),
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

    pub fn switch_to_uart(&mut self) {
        // Drop gpios before initalizing uart.
        drop(self.in_cq.take());
        drop(self.out_cq.take());

        let mut config = usart::Config::default();
        config.baudrate = 38_400; //TODO: COM2 for the moment but fix it!
        config.data_bits = usart::DataBits::DataBits8;
        config.stop_bits = usart::StopBits::STOP1;
        config.parity = usart::Parity::ParityEven;
        config.detect_previous_overrun = true;
        config.assume_noise_free = false;
        config.rx_pull = Pull::None;

        self.uart = Some(Uart::new_blocking(
            self.uart_instance.take().unwrap(),
            self.rx_pin.take().unwrap(),
            self.tx_pin.take().unwrap(),
            config,
        ).unwrap());

        self.mode = Mode::Uart;
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }
}

impl<'a> l6360::HardwareAccess for L6360_HW<'a> {
    fn enl_plus(&mut self, level: l6360::PinState) {
        match level {
            l6360::PinState::High => self.enl_plus.set_level(Level::High),
            l6360::PinState::Low => self.enl_plus.set_level(Level::Low),
        }
    }

    fn en_cq(&mut self, level: l6360::PinState) {
        match level {
            l6360::PinState::High => self.en_cq.set_level(Level::High),
            l6360::PinState::Low => self.en_cq.set_level(Level::Low),
        }
    }

    fn in_cq(&mut self, level: l6360::PinState) {
        let pin = self.in_cq.as_mut().unwrap();
        match level {
            l6360::PinState::High => pin.set_level(Level::High),
            l6360::PinState::Low => pin.set_level(Level::Low),
        }
    }

    fn out_cq(&self) -> l6360::PinState {
        match self.out_cq.as_ref().unwrap().get_level() {
            Level::High => l6360::PinState::High,
            Level::Low => l6360::PinState::Low,
        }
    }

    async fn exchange(&mut self, data: &[u8], answer: &mut [u8]) {
        self.en_cq(l6360::PinState::High);
        self.uart.as_mut().unwrap().blocking_write(data).unwrap();
        self.uart.as_mut().unwrap().blocking_flush().unwrap();

        self.en_cq(l6360::PinState::Low);
        self.uart.as_mut().unwrap().blocking_read(answer).unwrap();

        for byte in answer{
            info!("answer: {:#04x}", byte);
        }
    }
}
