#![deny(unsafe_code)]
#![no_main]
#![cfg_attr(not(test), no_std)]

use defmt::*;
use embassy_executor::{Spawner, main, task};
use embassy_stm32::gpio::{Output, Input, Level, Speed, Pull};
use embassy_stm32::i2c::{self, I2c};
use embassy_stm32::mode::Async;
use embassy_stm32::bind_interrupts;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::Instant;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use l6360::{self, L6360, Uart};
use iol::master;

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod l6360_uart;
use l6360_uart::L6360_Uart;

static L6360: Mutex<CriticalSectionRawMutex, Option<L6360<I2c<Async>, L6360_Uart, Output>>> = Mutex::new(None);

#[derive(Copy, Clone)]
struct MasterActions;

impl master::Actions for MasterActions {
    async fn wait_us(&self, duration: u64) {
        Timer::after_micros(duration).await;
    }

    async fn wait_ms(&self, duration: u64) {
        Timer::after_millis(duration).await;
    }

    // Note: This function should haven another name, so it is clear what exactly the output stage shall be configured to.
    async fn cq_output(&self, state: master::CqOutputState) {
        if let Some(l6360) = L6360.lock().await.as_mut() {

            l6360.uart.in_cq(l6360::PinState::High); // TODO: Note: so the output stays low. Think where to do it.
            let _ = l6360.set_cq_out_stage_configuration(l6360::CqOutputStageConfiguration::PushPull).await; //TODO: set to the correct value

            match state {
                master::CqOutputState::Disable => {
                    info!("disable cq output");
                    l6360.pins.en_cq.set_low();
                }
                master::CqOutputState::Enable => {
                    info!("enable cq output");
                    l6360.pins.en_cq.set_high();
                }
            }
        }
    }

    async fn get_cq(&self) -> master::PinState {
        if let Some(l6360) = L6360.lock().await.as_mut() {
            // Note: For a reason I don't understand yet, embassy does not use PinState.
            // Note: The l6360 inverts the state of C/Q.
            match l6360.uart.out_cq() {
                l6360::PinState::High => return master::PinState::Low,
                l6360::PinState::Low => return master::PinState::High,
            }
        }
        crate::panic!("couldn't access L6360");
    }

    async fn do_ready_pulse(&self) {
        if let Some(l6360) = L6360.lock().await.as_mut() {
            // Note: The l6360 inverts the state of C/Q.
            l6360.uart.in_cq(l6360::PinState::Low);

            // Busy waiting as we have to be very fast. This could be done nicer.
            let mut count = 0;
            while count < 36 {
                count += 1;
            }

            l6360.uart.in_cq(l6360::PinState::High);
            return;
        }
        crate::panic!("couldn't access L6360");
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

    async fn await_event_with_timeout_ms<F, T>(&self, duration: u64, future: F) -> Option<T>
    where
        F: core::future::Future<Output = T> + Send
    {
        embassy_time::with_timeout(embassy_time::Duration::from_millis(duration), future).await.ok()
    }

    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> master::ReadyPulseResult {
        if let Some(l6360) = L6360.lock().await.as_mut() {
            let result = embassy_time::with_timeout(
                embassy_time::Duration::from_millis(duration),
                measure_ready_pulse(l6360.uart.out_cq_.as_mut().unwrap())
            ).await;

            match result {
                Ok(_) => master::ReadyPulseResult::ReadyPulseOk,
                Err(_) => master::ReadyPulseResult::TimeToReadyElapsed,
            }
        }
        else {
            crate::panic!("Lock to L6360 failed"); //TODO: why is crate:: necessary here?
        }
    }

    async fn exchange_data(&self, data: &[u8], answer: &mut [u8]) {
        if let Some(l6360) = L6360.lock().await.as_mut() {
            if l6360.uart.get_mode() != l6360_uart::Mode::Uart {
                l6360.uart.switch_to_uart();
            }
            let _ = l6360.uart.exchange(data, answer); //TODO: set correct data
        }
        else {
            crate::panic!("Lock to L6360 failed"); //TODO: why is crate:: necessary here?
        }
    }
}

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

    *L6360.lock().await = Some(L6360::new(i2c, uart, 0b1100_000, pins, config).unwrap());

    let mut l6360_ref = L6360.lock().await;
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

async fn measure_ready_pulse(pin: &mut Input<'static>) {
    // // Note:
    // // This implementation of recognizing the Ready-Pulse is not maximaly accurate.
    // // On high load the pulse would not be measured accurately.
    // // It would be better to measure the pulse length directly with a timer.
    // // Currently PA10 is used which does not offer this possibility.
    // // The STEVAL-IOM001V1 would with minor changes also allow to use PA1 where it should be possible.

    // // Note: This solution is too slow due to context switches. A pulse of 750us is measured as around 915us.
    // // Incoming signals are inverted by the L6360
    // info!("waiting for pulse...");
    // pin.wait_for_falling_edge().await;
    // let start = Instant::now();
    // // Note: info! messages here would take too much time.
    // pin.wait_for_rising_edge().await;
    // let end = Instant::now();
    // info!("pulse received");

    // Note: Busy-Waiting for more accuracy. TODO: Better solution.
    info!("waiting for ready-pulse...");
    while pin.is_high() {}
    let start = Instant::now();
    while pin.is_low() {}
    let end = Instant::now();
    info!("ready-pulse received");

    let high_time_us = (end - start).as_micros();
    info!("Pin was high for {} us", high_time_us);
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
