use defmt::*;
use embassy_time::Timer;
use embassy_time::Instant;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use core::time::Duration;

use iol::master;
use l6360::{self, L6360, HardwareAccess};

use crate::l6360_hw::{self, L6360_HW};
use super::IOL_TRANSCEIVER;

#[derive(Copy, Clone)]
pub struct MasterActions;

impl master::Actions for MasterActions {
    async fn wait(&self, duration: Duration) {
        Timer::after(embassy_time::Duration::from_nanos(duration.as_nanos() as u64)).await;
    }

    async fn get_cq(&self) -> l6360::PinState {
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            l6360.hw.en_cq(l6360::PinState::Low);
            Timer::after_nanos(500).await; // Typical value is 225ns.

            // The L6360 inverts the pin value
            match l6360.hw.out_cq() {
                l6360::PinState::Low => l6360::PinState::High,
                l6360::PinState::High => l6360::PinState::Low,
            }
        }
        else {
            crate::panic!("Lock to L6360 failed");
        }
    }

    async fn wake_up_pulse(&self, direction: master::WakeUpPulseDirection) {
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            fn wait_blocking() {
                // Busy waiting as we have to be very fast. This could be done nicer.
                let mut count = 0;
                while count < 35 {
                    count += 1;
                }
            }

            // Note: The l6360 inverts the state of C/Q.
            // Execution:
            // - set cq value
            // - enable cq stage
            // - wait
            // - reset cq value
            match direction {
                master::WakeUpPulseDirection::Up => {
                    l6360.hw.in_cq(l6360::PinState::Low);
                    l6360.hw.en_cq(l6360::PinState::High);
                    wait_blocking();
                    l6360.hw.in_cq(l6360::PinState::High);
                }
                master::WakeUpPulseDirection::Down => {
                    l6360.hw.in_cq(l6360::PinState::High);
                    l6360.hw.en_cq(l6360::PinState::High);
                    wait_blocking();
                    l6360.hw.in_cq(l6360::PinState::Low);
                }
            }
        }
        else {
            crate::panic!("Lock to L6360 failed");
        }
    }

    async fn port_power_on(&self) {
        info!("port power on ...");
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            l6360.hw.enl_plus(l6360::PinState::High);
        }
        info!("done");
    }

    async fn port_power_off(&self) {
        info!("port power off ...");
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            l6360.hw.enl_plus(l6360::PinState::Low);
        }
        info!("done");
    }

    async fn await_event_with_timeout<F, T>(&self, duration: Duration, future: F) -> Option<T>
    where
        F: core::future::Future<Output = T> + Send
    {
        embassy_time::with_timeout(embassy_time::Duration::from_nanos(duration.as_nanos() as u64), future).await.ok()
    }

    async fn await_ready_pulse_with_timeout(&self, duration: Duration) -> master::ReadyPulseResult {
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            let result = embassy_time::with_timeout(
                embassy_time::Duration::from_nanos(duration.as_nanos() as u64),
                measure_ready_pulse(l6360),
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
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            if l6360.hw.get_mode() != l6360_hw::Mode::Uart {
                l6360.hw.switch_to_uart();
            }
            l6360.hw.exchange(data, answer).await;
        }
        else {
            crate::panic!("Lock to L6360 failed"); //TODO: why is crate:: necessary here?
        }
    }
}


async fn measure_ready_pulse(l6360: &mut L6360<I2c<'static, Async>, L6360_HW<'static>>) {
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
    while l6360.hw.out_cq() == l6360::PinState::High {}
    let start = Instant::now();
    while l6360.hw.out_cq() == l6360::PinState::Low {}
    let end = Instant::now();
    info!("ready-pulse received");

    let high_time_us = (end - start).as_micros();
    info!("Pin was high for {} us", high_time_us);
}
