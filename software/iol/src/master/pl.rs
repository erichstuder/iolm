//! Physical Layer
//!
//! see [#5](../../spec/IOL-Interface-Spec_10002_V114_Jun24.pdf#page=41)

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

pub use embedded_hal::digital::PinState;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub enum Service {
    //PL_SetMode,
    #[allow(non_camel_case_types)]
    PL_WakeUp,
    #[allow(non_camel_case_types)]
    PL_Transfer {
        data: [u8; 32],
        data_length: usize,
        answer_length: usize
    },
}

#[derive(PartialEq, Debug)]
pub enum ServiceResult {
    #[allow(non_camel_case_types)]
    PL_WakeUp,
    #[allow(non_camel_case_types)]
    PL_Transfer{ answer: [u8; 32] },
}

pub enum WakeUpPulseDirection {
    Up,
    Down,
}

pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait_us(&self, duration: u64);

    #[allow(async_fn_in_trait)]
    async fn get_cq(&self) -> PinState;

    #[allow(async_fn_in_trait)]
    async fn wake_up_pulse(&self, direction: WakeUpPulseDirection);

    #[allow(async_fn_in_trait)]
    async fn exchange_data(&self, data: &[u8], answer: &mut [u8]);
}

pub static SERVICE_CHANNEL: Channel<CriticalSectionRawMutex, Service, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, ServiceResult, 1> = Channel::new();

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
            match SERVICE_CHANNEL.receive().await {
                Service::PL_WakeUp => self.wake_up().await,
                Service::PL_Transfer { data, data_length, answer_length } => { self.transfer(&data[0..data_length], answer_length).await; }
            }
        }
    }

    pub async fn wake_up(&mut self) {
        #[allow(non_upper_case_globals)]
        const T_WU_us: u64 = 20;
        #[allow(non_upper_case_globals)]
        const T_REN_us: u64 = 500;

        let wake_up_pulse_direction = match self.actions.get_cq().await {
            PinState::Low => WakeUpPulseDirection::Up,
            PinState::High => WakeUpPulseDirection::Down,
        };

        self.actions.wake_up_pulse(wake_up_pulse_direction).await;

        self.actions.wait_us(T_REN_us - T_WU_us).await;

        RESULT_CHANNEL.send(ServiceResult::PL_WakeUp).await;
    }

    pub async fn transfer(&mut self, data: &[u8], answer_length: usize) {
        let mut answer = [0u8; 32];
        self.actions.exchange_data(data, &mut answer[0..answer_length]).await;
        info!("reading done");
        RESULT_CHANNEL.send(ServiceResult::PL_Transfer { answer }).await;
    }
}
