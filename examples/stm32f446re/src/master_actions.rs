use defmt::*;
use embassy_time::Timer;
use embassy_time::Instant;
use embassy_stm32::gpio::Input;

use iol::master;
use l6360::{self, Uart};

use crate::l6360_uart;
use super::IOL_TRANSCEIVER;

#[derive(Copy, Clone)]
pub struct MasterActions;

impl master::Actions for MasterActions {
    async fn wait_us(&self, duration: u64) {
        Timer::after_micros(duration).await;
    }

    async fn wait_ms(&self, duration: u64) {
        Timer::after_millis(duration).await;
    }

    // Note: This function should haven another name, so it is clear what exactly the output stage shall be configured to.
    async fn cq_output(&self, state: master::CqOutputState) {
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {

            l6360.uart.in_cq(l6360::PinState::High); // TODO: Note: so the output stays low. Think where to do it.
            let _ = l6360.set_cq_out_stage_configuration(l6360::CqOutputStageConfiguration::PushPull).await; //TODO: set to the correct value

            match state {
                master::CqOutputState::Disable => {
                    info!("disable cq output");
                    l6360.uart.en_cq.set_low();
                }
                master::CqOutputState::Enable => {
                    info!("enable cq output");
                    l6360.uart.en_cq.set_high();
                }
            }
        }
    }

    async fn get_cq(&self) -> master::PinState {
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
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
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
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
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            l6360.pins.enl_plus.set_high();
        }
        info!("done");
    }

    async fn port_power_off(&self) {
        info!("port power off ...");
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
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
        if let Some(l6360) = IOL_TRANSCEIVER.lock().await.as_mut() {
            let result = embassy_time::with_timeout(
                embassy_time::Duration::from_millis(duration),
                measure_ready_pulse(l6360.uart.out_cq.as_mut().unwrap())
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
            if l6360.uart.get_mode() != l6360_uart::Mode::Uart {
                l6360.uart.switch_to_uart();
            }
            l6360.uart.exchange(data, answer).await;
        }
        else {
            crate::panic!("Lock to L6360 failed"); //TODO: why is crate:: necessary here?
        }
    }
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
