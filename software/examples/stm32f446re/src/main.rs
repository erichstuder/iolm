#![deny(unsafe_code)]
#![no_main]
#![cfg_attr(not(test), no_std)]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Output, Level, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use l6360;
use iol::master::{self, smi};

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod iol_transceiver;
use iol_transceiver::IOL_Transceiver;

mod iol_master_actions;
use iol_master_actions::MasterActions;

// static IOL_TRANSCEIVER: Mutex<CriticalSectionRawMutex, Option<L6360<I2c<Async>>>> = Mutex::new(None);
static IOL_TRANSCEIVER: Mutex<CriticalSectionRawMutex, Option<IOL_Transceiver>> = Mutex::new(None);

#[main]
async fn main(spawner: Spawner) {
    // let p = embassy_stm32::init(Default::default());
    // heartbeat_led(spawner, p.PA5);
    // *IOL_TRANSCEIVER.lock().await = Some(L6360::new(i2c, 0b1100_000, config).unwrap());

    setup_hardware(spawner).await;

    // initialize iol transceiver
    let mut iol_transceiver_ref = IOL_TRANSCEIVER.lock().await;
    let iol_transceiver = iol_transceiver_ref.as_mut().unwrap();
    //iol_transceiver.init().await.unwrap();

    // set some blink pattern (just for fun)
    iol_transceiver.set_led_pattern(l6360::Led::LED1, 0xFFF0).await;
    iol_transceiver.set_led_pattern(l6360::Led::LED2, 0x000F).await;

    // power the connected iol-device
    iol_transceiver.enl_plus(l6360::PinState::High);
    //spawner.spawn(measure_ready_pulse(l6360.pins.out_cq)).unwrap();
    drop(iol_transceiver_ref);

    // setup iol stack and run
    let master = master::Master::new(MasterActions);
    spawner.spawn(run_master(master)).unwrap();

    // test code
    Timer::after_millis(2_000).await;
    info!("startup");
    let client_id = 0;
    let port_number = 0;
    let _ = smi::SMI_PortConfiguration(
        client_id,
        port_number,
        smi::annex_e::PortConfigList {
            port_mode: 0,
            validation_and_backup: 0,
            iq_behavior: 0,
            port_cycle_time: 0,
            vendor_id: 0,
            device_id: 0,
        }
    ).await;
    Timer::after_millis(100_000).await;
}

#[task]
async fn run_master(mut master: master::Master<MasterActions>) {
    info!("run master");
    master.run().await;
}

fn heartbeat_led(spawner: Spawner, pin: peripherals::PA5) {
    let led = Output::new(pin, Level::High, Speed::Low);
    spawner.spawn(blink(led)).unwrap();

    #[task]
    async fn blink(mut led: Output<'static>) -> ! {
        loop {
            //info!("high");
            led.set_high();
            Timer::after_millis(2000).await;

            //info!("low");
            led.set_low();
            Timer::after_millis(2000).await;
        }
    }
}

async fn setup_hardware(spawner: Spawner) {
    // let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(Default::default());
    // //use embassy_stm32::rcc;
    // let d = rcc::frequency::<peripherals::USART1>();

    // use embassy_stm32::rcc;
    // let _ = rcc::frequency::<peripherals::RCC>();

    // let clocks = embassy_stm32::rcc::cl();
    //info!("SYSCLK: {:?}", config.rcc.sys);
    // info!("HCLK: {:?}", p.rcc.clocks.hclk);
    // info!("PCLK1: {:?}", p.rcc.clocks.pclk1);
    // info!("PCLK2: {:?}", p.rcc.clocks.pclk2);

    heartbeat_led(spawner, p.PA5);

    bind_interrupts!(struct I2cIrqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    });

    let i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        I2cIrqs,
        p.DMA1_CH6,
        p.DMA1_CH0,
        Hertz(400_000),
        {
            let mut i2c_config = i2c::Config::default();
            i2c_config.sda_pullup = true;
            i2c_config.scl_pullup = true;
            i2c_config.timeout = embassy_time::Duration::from_millis(1000);
            i2c_config
        },
    );

    let iol_transceiver = IOL_Transceiver::new(i2c, p.USART1, p.PA9, p.PA10, p.PA6, p.PC0).await;

    // let config = l6360::Config {
    //     configuration_register: l6360::ConfigurationRegister {
    //         cq_output_stage_configuration: l6360::CqOutputStageConfiguration::PushPull,
    //     },
    //     control_register_1: l6360::ControlRegister1 {
    //         en_cgq_cq_pull_down: l6360::EN_CGQ_CQ_PullDown::ON_IfEnCq0,
    //     }
    // };

    // *IOL_TRANSCEIVER.lock().await = Some(L6360::new(i2c, 0b1100_000, config).unwrap());
    *IOL_TRANSCEIVER.lock().await = Some(iol_transceiver);

    // Note: This is necessary to make sure interrupts are enabled as I use my own IRQ-Handler for USART1.
    #[allow(unsafe_code)]
    unsafe {
        cortex_m::interrupt::enable();
    }
}
