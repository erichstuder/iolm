#![no_main]
#![cfg_attr(not(test), no_std)]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Output, Level, Speed, Pull};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::Instant;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use l6360::{self, L6360};
use iol::master;

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

static L6360: Mutex<CriticalSectionRawMutex, Option<L6360<I2c<'static, Async>, Output, ExtiInput>>> = Mutex::new(None);

#[derive(Copy, Clone)]
struct MasterActions;

impl master::Actions for MasterActions {
    async fn wait_ms(&self, duration: u64) {
        Timer::after_micros(duration).await;
    }

    async fn port_power_on(&self) {
        info!("port power on ...");
        if let Some(l6360) = L6360.lock().await.as_mut() {
            l6360.pins.enl_plus.set_high();
        }
        info!("done");
    }

    async fn port_power_off(&self) {
        info!("port power off ...");
        if let Some(l6360) = L6360.lock().await.as_mut() {
            l6360.pins.enl_plus.set_low();
        }
        info!("done");
    }

    async fn await_with_timeout_ms<F, T>(&self, future: F, duration: u64) -> Option<T>
    where
        F: core::future::Future<Output = T> + Send
    {
        embassy_time::with_timeout(embassy_time::Duration::from_millis(duration), future).await.ok()
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let led = Output::new(peripherals.PA5, Level::High, Speed::Low);

    spawner.spawn(blink(led)).unwrap();

    bind_interrupts!(struct Irqs {
        I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
        I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    });

    let i2c = I2c::new(
        peripherals.I2C1,
        peripherals.PB8,
        peripherals.PB9,
        Irqs,
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

    let pins = l6360::Pins {
        enl_plus: Output::new(peripherals.PA6, Level::Low, Speed::Low),
        out_cq: ExtiInput::new(peripherals.PA10, peripherals.EXTI10, Pull::None),
    };

    let config = l6360::Config {
        control_register_1: l6360::ControlRegister1 {
            en_cgq_cq_pulldown: l6360::EN_CGQ_CQ_PullDown::ON_IfEnCq0,
        }
    };

    *L6360.lock().await = Some(L6360::new(i2c, 0b1100_000, pins, config).unwrap());
    let mut l6360_ref = L6360.lock().await;
    let l6360 = l6360_ref.as_mut().unwrap();

    // let mut l6360 = L6360::new(i2c, 0b1100_000, pins, config).unwrap();
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
async fn measure_ready_pulse(mut pin: ExtiInput<'static>) -> ! {
    loop {
        if pin.is_high() {
            info!("pin is high");
        }
        else {
            info!("pin is low");
        }

        // Note:
        // This implementation of recognizing the Ready-Pulse is not maximaly accurate.
        // On high load the pulse would not be measured accurately.
        // It would be better to measure the pulse length directly with a timer.
        // Currently PA10 is used which does not offer this possibility.
        // The STEVAL-IOM001V1 would with minor changes also allow to use PA1 where it should be possible.

        // Note:
        // Incoming signals are inverted by the L6360
        info!("waiting for pulse...");
        pin.wait_for_falling_edge().await;
        let start = Instant::now();
        // Note: info! messages here would take too much time.
        pin.wait_for_rising_edge().await;
        let end = Instant::now();
        info!("pulse received");

        let high_time_us = (end - start).as_micros();
        info!("Pin was high for {} us", high_time_us);
    }
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
