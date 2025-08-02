use defmt::info;
use embassy_stm32::bind_interrupts;
use embassy_stm32::i2c::I2c;
use embassy_stm32::usart::{self, Uart};
use embassy_stm32::peripherals;
use embassy_stm32::gpio::{Output, Input, Level, Speed, Pull};
use embassy_stm32::Peripheral;
use embassy_stm32::mode::Async;
use embassy_time::{Duration, Instant, with_timeout};

use embassy_stm32::interrupt::InterruptExt;
use embassy_stm32::interrupt;
use embassy_stm32::pac;
use embassy_stm32::rcc;
use core::sync::atomic::{AtomicUsize, Ordering};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use l6360::{L6360, Led};
pub use l6360::PinState;

// bind_interrupts!(struct I2cIrqs {
//     I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
//     I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
// });

// bind_interrupts!(struct UartIrqs {
//     USART1 => usart::InterruptHandler<peripherals::USART1>;
// });

static mut TX_BUF: [u8; 32] = [0; 32];
static mut RX_BUF: [u8; 32] = [0; 32];
static TX_LEN: AtomicUsize = AtomicUsize::new(0);
static RX_LEN: AtomicUsize = AtomicUsize::new(0);
static TX_BUF_INDEX: AtomicUsize = AtomicUsize::new(0);
static RX_BUF_INDEX: AtomicUsize = AtomicUsize::new(0);
static RX_DONE_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static mut DURATION_SEND_AND_RECEIVE: Duration = Duration::from_micros(0);
static mut EN_CQ: Option<Output<'static>> = None;
static mut FINAL_TIME: Instant = Instant::from_secs(0);

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
    // en_cq: Output<'a>,
    in_cq: Option<Output<'a>>,
    out_cq: Option<Input<'a>>,
    uart: Option<Uart<'a, Async>>,
    mode: Mode,
}

impl<'a> IOL_Transceiver<'a> {
    pub async fn new(
        i2c: I2c<'a, Async>,
        uart_instance: peripherals::USART1, //idea: take it, so no one else can
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

        // TODO: Is there another solution than using unsafe?
        #[allow(unsafe_code)]
        unsafe {
            EN_CQ = Some(Output::new(en_cq, Level::Low, Speed::Low));
        }

        Self {
            l6360,
            uart_instance: Some(uart_instance),
            tx_dma: Some(tx_dma),
            rx_dma: Some(rx_dma),
            tx_pin: Some(tx_pin),
            rx_pin: Some(rx_pin),
            enl_plus: Output::new(enl_plus, Level::Low, Speed::Low),
            // en_cq: Output::new(en_cq, Level::Low, Speed::Low),
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

        // self.uart = Some(Uart::new(
        //     self.uart_instance.take().unwrap(),
        //     self.rx_pin.take().unwrap(),
        //     self.tx_pin.take().unwrap(),
        //     UartIrqs,
        //     self.tx_dma.take().unwrap(),
        //     self.rx_dma.take().unwrap(),
        //     config,
        // ).unwrap());

        self.mode = Mode::Uart;


        let rcc = pac::RCC;


        rcc.apb2enr().modify(|w| w.set_usart1en(true));
        rcc.ahb1enr().modify(|w| {
            w.set_gpioaen(true);
            // w.set_dma2en(true);  // Enable DMA2 clock
        });

        let gpioa = pac::GPIOA;

        gpioa.moder().modify(|w| {
            w.set_moder(9, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE);  // PA9 = Alternate function
            w.set_moder(10, embassy_stm32::pac::gpio::vals::Moder::ALTERNATE); // PA10 = Alternate function
        });

        gpioa.afr(1).modify(|w| {
            w.set_afr(9-8, 7);   // PA9 = AF7
            w.set_afr(10-8, 7);  // PA10 = AF7
        });



        let uart = pac::USART1;

        uart.cr1().modify(|w| {
            w.set_m0(embassy_stm32::pac::usart::vals::M0::BIT9);
            w.set_pce(true);
            w.set_ps(embassy_stm32::pac::usart::vals::Ps::EVEN);
            w.set_te(true);
            w.set_re(true);
            //w.set_rxneie(true);
            //w.set_tcie(true);
        });

        uart.cr2().modify(|w| {
            w.set_stop(embassy_stm32::pac::usart::vals::Stop::STOP1);
        });

        uart.cr3().modify(|w| {
            w.set_eie(true);
            w.set_dmat(true);  // Enable DMA for transmission
        });

        uart.cr1().modify(|w| w.set_ue(true));


        // // Add DMA configuration here:
        // let dma2 = pac::DMA2;
        // // Configure DMA2 Stream 7 for USART1_TX
        // dma2.st(7).cr().modify(|w| w.set_en(false));
        // while dma2.st(7).cr().read().en() {}

        // dma2.st(7).cr().write(|w| {
        //     w.set_chsel(4);
        //     w.set_dir(embassy_stm32::pac::dma::vals::Dir::MEMORY_TO_PERIPHERAL);
        //     w.set_minc(true);
        //     w.set_msize(embassy_stm32::pac::dma::vals::Size::BITS8);
        //     w.set_psize(embassy_stm32::pac::dma::vals::Size::BITS8);
        //     w.set_tcie(true);
        // });

        // dma2.st(7).par().write_value(0x4001_1004);

        // #[allow(unsafe_code)]
        // unsafe { interrupt::DMA2_STREAM7.enable(); }


        // Enable USART1 interrupt in NVIC
        interrupt::USART1.set_priority(interrupt::Priority::P6); // TODO: what priority is necessary?
        #[allow(unsafe_code)]
        unsafe { interrupt::USART1.enable(); }
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

    pub fn en_cq(&mut self, level: l6360::PinState) {// TODO: Is there a better solution than using unsafe?
        #[allow(unsafe_code)]
        match level {
        //     l6360::PinState::High => self.en_cq.set_level(Level::High),
        //     l6360::PinState::Low => self.en_cq.set_level(Level::Low),

            l6360::PinState::High => unsafe { EN_CQ.as_mut().unwrap().set_level(Level::High) },
            l6360::PinState::Low => unsafe { EN_CQ.as_mut().unwrap().set_level(Level::Low) },
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
        //self.uart.as_ref().unwrap().set_baudrate(baudrate).unwrap();
        let uart_freq = rcc::frequency::<peripherals::USART1>();

        let brr_value = (uart_freq.0 + baudrate / 2) / baudrate;

        let uart = pac::USART1;
        uart.brr().write(|w| w.set_brr(brr_value as u16));
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

    pub async fn send_and_receive(&mut self, data: &[u8], answer: &mut[u8]) -> Result<(), iol::master::TransferError>{
        if data.len() == 0 {
            panic!("Currently Length 0 is not allowed.");
            // Note: If this shall be allowed special care might be taken.
        }

        #[allow(unsafe_code)]
        unsafe {
            TX_BUF[..data.len()].copy_from_slice(data);
        }

        TX_BUF_INDEX.store(0, Ordering::Relaxed);
        TX_LEN.store(data.len(), Ordering::Release);

        let uart = pac::USART1;
        if !uart.sr().read().txe() {
            panic!("The transmit data register should always be empty here!");
        }
        self.en_cq(l6360::PinState::High);
        uart.dr().write(|w| w.set_dr(data[0] as u16));
        let start_time = embassy_time::Instant::now();

        uart.cr1().modify(|w| {
            w.set_txeie(true);
        });

        RX_BUF_INDEX.store(0, Ordering::Relaxed);
        RX_LEN.store(answer.len(), Ordering::Release);
        // loop {
        //     if RX_DONE_SIGNAL.signaled() {
        //         info!("siggggggggggggggggggggg");
        //         break;
        //     }
        //     embassy_time::Timer::after(Duration::from_millis(1000)).await;
        //     info!("check if signaled");
        // }

        // info!("wait on signal");
        RX_DONE_SIGNAL.wait().await;
        info!("signal receiveeeeeeeeeeeeeeeeeeed");

        #[allow(unsafe_code)]
        unsafe {
            let rx_len = RX_BUF_INDEX.load(Ordering::Relaxed) + 1;
            answer[..rx_len].copy_from_slice(&RX_BUF[..rx_len]);
        }

        //DEBUG
        let rx_len = RX_BUF_INDEX.load(Ordering::Relaxed) + 1;
        info!("Received {} bytes:", rx_len);
        for (i, &byte) in answer[..rx_len].iter().enumerate() {
            info!("answer[{}]: {:#04x}", i, byte);
        }

        #[allow(unsafe_code)]
        let end_time = unsafe { FINAL_TIME };
        let duration = end_time.duration_since(start_time);
        info!("Send and receive took: {} microseconds", duration.as_micros());


        // let uart = pac::USART1;
        // for &byte in data {
        //     while !uart.sr().read().txe() {}
        //     uart.dr().write(|w| w.set_dr(byte as u16));
        // }
        // Wait for transmission complete
        //while !uart.sr().read().tc() {}
        //self.uart.as_mut().unwrap().blocking_write(data).map_err(Self::convert_uart_error)?;
        //self.uart.as_mut().unwrap().blocking_flush().map_err(Self::convert_uart_error)?;

        Ok(())
    }

    // pub async fn try_receive(&mut self, answer: &mut Option<&mut [u8]>) -> Result<(), iol::master::TransferError>{
    //     self.en_cq(l6360::PinState::Low);
    //     if let Some(buffer) = answer {
    //         match embassy_time::with_timeout(
    //             Duration::from_millis(1),
    //             self.uart.as_mut().unwrap().read(buffer)
    //         ).await {
    //             Ok(Ok(())) => {
    //                 info!("ok");
    //                 for byte in buffer.iter(){
    //                     info!("answer: {:#04x}", byte);
    //                 }
    //                 Ok(())
    //             }
    //             Ok(Err(e)) => {
    //                 info!("uart error: {:?}", e);
    //                 Err(Self::convert_uart_error(e))
    //             }
    //             Err(_) => {
    //                 info!("No answer for now");
    //                 *answer = None;
    //                 Ok(())
    //             }
    //         }
    //     } else {
    //         panic!("must be Some");
    //     }
    // }

}

// #[interrupt]
// fn DMA2_STREAM7() {
//     let dma2 = pac::DMA2;

//     // Check if transfer complete
//     if dma2.isr(1).read().tcif(7) {
//         // Clear transfer complete flag
//         dma2.ifcr(1).write(|w| w.set_tcif(7, true));
//         // Signal completion
//         // DMA_TX_COMPLETE.store(true, Ordering::Release);

//         info!("DMA TX transfer complete");
//     }

//     // Check for errors
//     if dma2.isr(1).read().teif(7) {
//         dma2.ifcr(1).write(|w| w.set_teif(7, true));
//         info!("DMA TX transfer error");
//     }
// }

#[interrupt]
fn USART1() {
    // Note: This is the potential end time. It is measured here to be as accurate as reasonable possible.
    #[allow(unsafe_code)]
    unsafe {
        FINAL_TIME = embassy_time::Instant::now();
    }

    let uart = pac::USART1;
    let sr = uart.sr().read();

    //info!("sr: {:#010x}", sr.0);

    if sr.tc() {
        // info!("tc");

        uart.cr1().modify(|w| {
            w.set_tcie(false);
            // w.set_re(true);
            w.set_rxneie(true);
        });

        // TODO: Is there another solution than using unsafe?
        #[allow(unsafe_code)]
        unsafe {
            EN_CQ.as_mut().unwrap().set_level(Level::Low)
        }
        uart.sr().modify(|w| w.set_tc(false));
        // let _ = uart.dr().read().dr();
    }
    // Check for transmission complete
    if sr.txe() {
        // info!("txe");
        let mut tx_buf_index = TX_BUF_INDEX.load(Ordering::Relaxed);
        let tx_len = TX_LEN.load(Ordering::Relaxed);


        // if tx_len == 0 {
        //     return; //debugggggggggggg
        // }

        if tx_buf_index < tx_len-1 {
            tx_buf_index += 1;

            //#[allow(unsafe_code)]
            //let dummy = unsafe{TX_BUF[tx_buf_index]};
            //info!("write byte {}", dummy);

            // TODO: is there a better solution than using unsafe?
            #[allow(unsafe_code)]
            unsafe {
                uart.dr().write(|w| w.set_dr(TX_BUF[tx_buf_index] as u16));
            }

            TX_BUF_INDEX.store(tx_buf_index, Ordering::Relaxed);
        }
        else {
            uart.cr1().modify(|w| {
                w.set_txeie(false);
                w.set_tcie(true);
            });
            uart.sr().modify(|w| w.set_txe(false));
        }
    }

    // Check for received data
    if sr.rxne() {
        //info!("rxne");
        let mut rx_buf_index = RX_BUF_INDEX.load(Ordering::Relaxed);
        let rx_len = RX_LEN.load(Ordering::Relaxed);

        // if rx_len == 0 {
        //     return; //debugggggggggggg
        // }

        let dr = uart.dr().read().dr() as u8;

        if rx_buf_index < rx_len {
            // TODO: is there a better solution than using unsafe?
            #[allow(unsafe_code)]
            unsafe {
                RX_BUF[rx_buf_index] = dr;
            }
        }

        if rx_buf_index < rx_len-1 {
            RX_BUF_INDEX.store(rx_buf_index+1, Ordering::Release);
        }

        if rx_buf_index >= rx_len-1 {
            #[allow(unsafe_code)]
            unsafe {
                // FINAL_TIME = embassy_time::Instant::now();
                EN_CQ.as_mut().unwrap().set_level(Level::High);
            }

            uart.cr1().modify(|w| {
                // w.set_re(false);
                w.set_rxneie(false);
            });
            RX_DONE_SIGNAL.signal(());
            info!("dummy");
        }


        // unsafe {
        //     if RX_COUNT < RX_BUFFER.len() {
        //         RX_BUFFER[RX_COUNT] = received_byte;
        //         RX_COUNT += 1;
        //     }
        // }

        // Signal that data is available
        // RX_DATA_READY.store(true, Ordering::Release);
        // RX_SIGNAL.signal(received_byte);

        //info!("USART1 RX----------------: {:#04x}", received_byte);
    }

    // Check for overrun error
    if sr.ore() {
        info!("USART1 Overrun error!");
        // Clear ORE by reading SR then reading DR
        let _dummy = uart.dr().read();
    }

    // Check for framing error
    if sr.fe() {
        info!("USART1 Framing error!");
        // Clear FE by reading SR then reading DR
        let _dummy = uart.dr().read();
    }

    // Check for parity error
    if sr.pe() {
        info!("USART1 Parity error!");
        let _ = uart.dr().read();
    }
}
