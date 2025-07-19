//! Physical Layer
//!
//! see [#5 - IO-Link Specification](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=41)

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

#[cfg(test)]
use mockall::automock;

use core::time::Duration;

pub use embedded_hal::digital::PinState;

mod pl_services;
pub use pl_services::outside_pl as services;
pub use pl_services::{Service, ServiceResult};
use pl_services::inside_dl::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum WakeUpPulseDirection {
    Up,
    Down,
}

#[cfg_attr(test, automock)]
pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait(&self, duration: Duration);

    #[allow(async_fn_in_trait)]
    async fn get_cq(&self) -> PinState;

    #[allow(async_fn_in_trait)]
    async fn wake_up_pulse(&self, direction: WakeUpPulseDirection);

    #[allow(async_fn_in_trait)]
    async fn exchange_data(&self, data: &[u8], answer: &mut [u8]);
}

pub struct PL<A: Actions> {
    actions: A,
}

impl<A: Actions> PL<A> {
    pub fn new(actions: A) -> Self {
        Self {
            actions,
        }
    }

    pub async fn run(&mut self) {
        loop {
            // For testability the loop body is in its own funtion.
            self.handle_service().await;
        }
    }

    async fn handle_service(&mut self) {
        match receive_service().await {
            Service::PL_WakeUp => self.wake_up().await,
            Service::PL_Transfer { data, data_length, answer_length } => { self.transfer(&data[0..data_length], answer_length).await; }
        }
    }

    async fn wake_up(&mut self) {
        #[allow(non_upper_case_globals)]
        const T_WU: Duration = Duration::from_micros(20);
        #[allow(non_upper_case_globals)]
        const T_REN: Duration = Duration::from_micros(500);

        let wake_up_pulse_direction = match self.actions.get_cq().await {
            PinState::Low => WakeUpPulseDirection::Up,
            PinState::High => WakeUpPulseDirection::Down,
        };

        self.actions.wake_up_pulse(wake_up_pulse_direction).await;

        self.actions.wait(T_REN - T_WU).await;

        send_service_result(ServiceResult::PL_WakeUp).await;
    }

    async fn transfer(&mut self, data: &[u8], answer_length: usize) {
        let mut answer = [0u8; 32];
        self.actions.exchange_data(data, &mut answer[0..answer_length]).await;
        info!("reading done");
        send_service_result(ServiceResult::PL_Transfer { answer }).await;
    }
}

// see Table 9
pub mod dynamic_characteristic_of_the_transmission{
    use core::time::Duration;

    const fn calculate_t_bit(f_dtr: u32) -> Duration {
        Duration::from_nanos((1e9f64 / (f_dtr as f64)) as u64)
    }

    pub mod com1 {
        use super::*;
        pub const F_DTR: u32 = 4800;
        pub const T_BIT: Duration = calculate_t_bit(F_DTR);
    }

    pub mod com2 {
        use super::*;
        pub const F_DTR: u32 = 38400;
        pub const T_BIT: Duration = calculate_t_bit(F_DTR);
    }

    pub mod com3 {
        use super::*;
        pub const F_DTR: u32 = 230400;
        pub const T_BIT: Duration = calculate_t_bit(F_DTR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use mockall::predicate::eq;

    #[tokio::test]
    async fn wake_up_service() {
        let mut mock_actions = MockActions::new();

        // Just check that the calls happen.
        mock_actions.expect_get_cq()
            .times(1)
            .returning(|| PinState::Low);

        mock_actions.expect_wake_up_pulse()
            .times(1)
            .returning(|_| ());

        mock_actions.expect_wait()
            .times(1)
            .returning(|_| ());

        let mut pl = PL::new(mock_actions);

        services::send_service(Service::PL_WakeUp).await;
        pl.handle_service().await;
        let result = services::receive_service_result().await;
        assert_eq!(result, ServiceResult::PL_WakeUp);
    }

    #[tokio::test]
    async fn transfer_service() {
        let mut mock_actions = MockActions::new();

        // Just check that the call happens.
        mock_actions.expect_exchange_data()
            .times(1)
            .returning(|_,_| ());

        let mut pl = PL::new(mock_actions);

        services::send_service(Service::PL_Transfer {
            data: [0u8; 32],
            data_length: 22,
            answer_length: 5,
        }).await;

        pl.handle_service().await;
        let result = services::receive_service_result().await;
        assert_eq!(result, ServiceResult::PL_Transfer { answer: [0u8; 32] });
    }

    #[tokio::test]
    async fn wake_up() {
        let test_cases: &[(
            PinState,       WakeUpPulseDirection)] = &[
         // cq              wake-up-pulse
         (  PinState::Low,  WakeUpPulseDirection::Up  ),
         (  PinState::High, WakeUpPulseDirection::Down),
        ];

        for (cq_pin_state, wake_up_pulse_direction) in test_cases {
            let mut mock_actions = MockActions::new();

            mock_actions.expect_get_cq()
                .times(1)
                .returning(|| *cq_pin_state);

            mock_actions.expect_wake_up_pulse()
                .times(1)
                .with(eq(*wake_up_pulse_direction))
                .returning(|_| ());

            mock_actions.expect_wait()
                .times(1)
                .with(eq(Duration::from_micros(480)))
                .returning(|_| ());

            let mut pl = PL::new(mock_actions);
            pl.wake_up().await;
            let _ = services::receive_service_result().await;
        }
    }

    #[tokio::test]
    async fn transfer() {
        let mut mock_actions = MockActions::new();

        let test_data: [u8; 10] = [9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        let test_answer_length: usize = 16;

        mock_actions.expect_exchange_data()
            .times(1)
            .withf(move |data, answer| {
                data == &test_data[..] &&
                answer.len() == test_answer_length
            })
            .returning(|_,_| ());

        let mut pl = PL::new(mock_actions);
        pl.transfer(&test_data, test_answer_length).await;
        let _ = services::receive_service_result().await;
    }
}
