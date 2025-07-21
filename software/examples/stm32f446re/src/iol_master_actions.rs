use defmt::*;
use embassy_time::Timer;
use embassy_time::Instant;
use core::time::Duration;

use crate::iol_transceiver::{self, IOL_Transceiver};
use super::IOL_TRANSCEIVER;

#[derive(Copy, Clone)]
pub struct MasterActions;

impl iol::master::Actions for MasterActions {
    async fn wait(&self, duration: Duration) {
        Timer::after(embassy_time::Duration::from_nanos(duration.as_nanos() as u64)).await;
    }

    async fn get_cq(&self) -> iol_transceiver::PinState {
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            iol_transceiver.en_cq(iol_transceiver::PinState::Low);
            Timer::after_nanos(500).await; // Typical value is 225ns.

            // The iol_transceiver inverts the pin value
            match iol_transceiver.out_cq() {
                iol_transceiver::PinState::Low => iol_transceiver::PinState::High,
                iol_transceiver::PinState::High => iol_transceiver::PinState::Low,
            }
        }
        else {
            crate::panic!("Lock to iol_transceiver failed");
        }
    }

    async fn wake_up_pulse(&self, direction: iol::master::WakeUpPulseDirection) {
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            fn wait_blocking() {
                // Busy waiting as we have to be very fast. This could be done nicer.
                let mut count = 0;
                while count < 35 {
                    count += 1;
                }
            }

            // Note: The iol_transceiver inverts the state of C/Q.
            // Execution:
            // - set cq value
            // - enable cq stage
            // - wait
            // - reset cq value
            match direction {
                iol::master::WakeUpPulseDirection::Up => {
                    iol_transceiver.in_cq(iol_transceiver::PinState::Low);
                    iol_transceiver.en_cq(iol_transceiver::PinState::High);
                    wait_blocking();
                    iol_transceiver.in_cq(iol_transceiver::PinState::High);
                }
                iol::master::WakeUpPulseDirection::Down => {
                    iol_transceiver.in_cq(iol_transceiver::PinState::High);
                    iol_transceiver.en_cq(iol_transceiver::PinState::High);
                    wait_blocking();
                    iol_transceiver.in_cq(iol_transceiver::PinState::Low);
                }
            }
        }
        else {
            crate::panic!("Lock to iol_transceiver failed");
        }
    }

    async fn port_power_on(&self) {
        info!("port power on ...");
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            iol_transceiver.enl_plus(iol_transceiver::PinState::High);
        }
        info!("done");
    }

    async fn port_power_off(&self) {
        info!("port power off ...");
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            iol_transceiver.enl_plus(iol_transceiver::PinState::Low);
        }
        info!("done");
    }

    async fn await_event_with_timeout<F, T>(&self, duration: Duration, future: F) -> Option<T>
    where
        F: core::future::Future<Output = T> + Send
    {
        embassy_time::with_timeout(embassy_time::Duration::from_nanos(duration.as_nanos() as u64), future).await.ok()
    }

    async fn await_ready_pulse_with_timeout(&self, duration: Duration) -> iol::master::ReadyPulseResult {
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            let result = embassy_time::with_timeout(
                embassy_time::Duration::from_nanos(duration.as_nanos() as u64),
                measure_ready_pulse(iol_transceiver),
            ).await;

            match result {
                Ok(_) => iol::master::ReadyPulseResult::ReadyPulseOk,
                Err(_) => iol::master::ReadyPulseResult::TimeToReadyElapsed,
            }
        }
        else {
            crate::panic!("Lock to iol_transceiver failed"); //TODO: why is crate:: necessary here?
        }
    }

    async fn set_baudrate(&self, baudrate: u32) {
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            if iol_transceiver.get_mode() != iol_transceiver::Mode::Uart {
                iol_transceiver.switch_to_uart();
            }
            iol_transceiver.set_baudrate(baudrate);
        }
    }

    async fn send_data(&self, data: &[u8]) -> Result<(), iol::master::TransferError> {
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            if iol_transceiver.get_mode() != iol_transceiver::Mode::Uart {
                iol_transceiver.switch_to_uart();
            }
            iol_transceiver.send(data)?;
        }
        else {
            crate::panic!("Lock to iol_transceiver failed"); //TODO: why is crate:: necessary here?
        }
        Ok(())
    }

    async fn try_receive_data(&self, answer: &mut Option<&mut [u8]>) -> Result<(), iol::master::TransferError> {
        if let Some(iol_transceiver) = IOL_TRANSCEIVER.lock().await.as_mut() {
            if iol_transceiver.get_mode() != iol_transceiver::Mode::Uart {
                iol_transceiver.switch_to_uart();
            }
            iol_transceiver.try_receive(answer).await?;
        }
        else {
            crate::panic!("Lock to iol_transceiver failed"); //TODO: why is crate:: necessary here?
        }
        Ok(())
    }
}


async fn measure_ready_pulse(iol_transceiver: &IOL_Transceiver<'static>) {
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
    while iol_transceiver.out_cq() == iol_transceiver::PinState::High {}
    let start = Instant::now();
    while iol_transceiver.out_cq() == iol_transceiver::PinState::Low {}
    let end = Instant::now();
    info!("ready-pulse received");

    let high_time_us = (end - start).as_micros();
    info!("Pin was high for {} us", high_time_us);
}
