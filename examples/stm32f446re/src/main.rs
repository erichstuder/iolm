#![deny(unsafe_code)]
#![no_main]
#![cfg_attr(not(test), no_std)]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Output, Level, Speed};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use l6360::{self, L6360};
use iol::master;

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod l6360_uart;
use l6360_uart::L6360_Uart;

mod master_actions;
use master_actions::MasterActions;

static L6360_: Mutex<CriticalSectionRawMutex, Option<L6360<I2c<Async>, L6360_Uart, Output>>> = Mutex::new(None);

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(peripherals.PA5, Level::High, Speed::Low);

    spawner.spawn(blink(led)).unwrap();

    bind_interrupts!(struct I2cIrqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    });

    let i2c = I2c::new(
        peripherals.I2C1,
        peripherals.PB8,
        peripherals.PB9,
        I2cIrqs,
        peripherals.DMA1_CH6,
        peripherals.DMA1_CH0,
        Hertz(400_000),
        {
            let mut i2c_config = i2c::Config::default();
            i2c_config.sda_pullup = true;
            i2c_config.scl_pullup = true;
            i2c_config.timeout = embassy_time::Duration::from_millis(1000);
            i2c_config
        },
    );


    let uart = L6360_Uart::new(peripherals.USART1, peripherals.PA9, peripherals.PA10);

    let pins = l6360::Pins {
        enl_plus: Output::new(peripherals.PA6, Level::Low, Speed::Low),
        en_cq: Output::new(peripherals.PC0, Level::Low, Speed::Low),
    };

    let config = l6360::Config {
        control_register_1: l6360::ControlRegister1 {
            en_cgq_cq_pulldown: l6360::EN_CGQ_CQ_PullDown::ON_IfEnCq0,
        }
    };

    *L6360_.lock().await = Some(L6360::new(i2c, uart, 0b1100_000, pins, config).unwrap());

    let mut l6360_ref = L6360_.lock().await;
    let l6360 = l6360_ref.as_mut().unwrap();
    l6360.init().await.unwrap();
    l6360.set_led_pattern(l6360::Led::LED1, 0xFFF0).await.unwrap();
    l6360.set_led_pattern(l6360::Led::LED2, 0x000F).await.unwrap();
    l6360.pins.enl_plus.set_high();
    //spawner.spawn(measure_ready_pulse(l6360.pins.out_cq)).unwrap();
    drop(l6360_ref);

    let (mut master, port_power_switching, dl) = master::Master::new(MasterActions);
    spawner.spawn(run_port_power_switching(port_power_switching)).unwrap();
    spawner.spawn(run_dl(dl)).unwrap();

    Timer::after_millis(2_000).await;

    info!("startup");
    master.dl_set_mode_startup().await;

    Timer::after_millis(100_000).await;
}

#[task]
async fn run_port_power_switching(mut port_power_switching: master::PortPowerSwitchingStateMachine<MasterActions>) {
    info!("run port power switching");
    port_power_switching.run().await;
}

#[task]
async fn run_dl(mut dl: master::DlModeHandlerStateMachine<MasterActions>) {
    info!("run dl");
    dl.run().await;
}

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
