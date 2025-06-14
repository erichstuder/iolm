use embassy_stm32::bind_interrupts;
use embassy_stm32::usart::{self, BufferedUart};
use embassy_stm32::peripherals;
use embassy_stm32::gpio::{Output, Input, Level, Speed, Pull};
use embassy_stm32::Peripheral;
use static_cell::StaticCell;
use embedded_io_async::{Write, Read};

bind_interrupts!(struct UartIrqs {
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
});

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Gpio,
    Uart,
}

// Note: bind_interrupts is not generic. This leads to having concrete types.
#[allow(non_camel_case_types)]
pub struct L6360_Uart<'a> {
    uart_instance: Option<peripherals::USART1>,
    tx_pin: Option<peripherals::PA9>,
    rx_pin: Option<peripherals::PA10>,
    pub in_cq_: Option<Output<'a>>, //TODO: rename
    pub out_cq_: Option<Input<'a>>, //TODO: rename
    uart: Option<BufferedUart<'a>>,
    mode: Mode,
}

impl<'a> L6360_Uart<'a> {
    pub fn new(uart_instance: peripherals::USART1, tx_pin: peripherals::PA9, rx_pin: peripherals::PA10) -> Self {
        // Note: This struct uses unsafe to be able to switch between gpio and uart.
        #[allow(unsafe_code)]
        let tx_clone = unsafe { tx_pin.clone_unchecked() };
        #[allow(unsafe_code)]
        let rx_clone = unsafe { rx_pin.clone_unchecked() };

        Self {
            uart_instance: Some(uart_instance),
            tx_pin: Some(tx_pin),
            rx_pin: Some(rx_pin),
            in_cq_: Some(Output::new(tx_clone, Level::Low, Speed::Low)),
            out_cq_: Some(Input::new(rx_clone, Pull::None)),
            uart: None,
            mode: Mode::Gpio,
        }
    }

    pub fn switch_to_uart(&mut self) {
        // Drop gpios before initalizing uart.
        drop(self.in_cq_.take());
        drop(self.out_cq_.take());

        static TX_BUF: StaticCell<[u8; 100]> = StaticCell::new();
        let tx_buf = TX_BUF.init([0u8; 100]);
        static RX_BUF: StaticCell<[u8; 100]> = StaticCell::new();
        let rx_buf = RX_BUF.init([0u8; 100]);

        let mut config = usart::Config::default();
        config.baudrate = 38_400; //TODO: COM2 for the moment but fix it!
        config.data_bits = usart::DataBits::DataBits8;
        config.stop_bits = usart::StopBits::STOP1;
        config.parity = usart::Parity::ParityEven;
        config.detect_previous_overrun = true;
        config.assume_noise_free = false;
        config.rx_pull = Pull::None;

        self.uart = Some(BufferedUart::new(
            self.uart_instance.take().unwrap(),
            UartIrqs,
            self.rx_pin.take().unwrap(),
            self.tx_pin.take().unwrap(),
            tx_buf,
            rx_buf,
            config,
        ).unwrap());

        self.mode = Mode::Uart;
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }
}

impl<'a> l6360::Uart for L6360_Uart<'a> {
    fn in_cq(&mut self, level: l6360::PinState) {
        match level {
            l6360::PinState::High => self.in_cq_.as_mut().unwrap().set_level(Level::High),
            l6360::PinState::Low => self.in_cq_.as_mut().unwrap().set_level(Level::Low),
        }
    }

    fn out_cq(&self) -> l6360::PinState {
        match self.out_cq_.as_ref().unwrap().get_level() {
            Level::High => l6360::PinState::High,
            Level::Low => l6360::PinState::Low,
        }
    }

    async fn exchange(&mut self, data: &[u8], answer: &mut [u8]) {
        let uart = self.uart.as_mut().unwrap();
        let _ = uart.write_all(data).await.unwrap();
        uart.read_exact(answer).await.unwrap();
    }
}
