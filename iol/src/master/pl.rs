// see #5.3.3.3

// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

pub use embedded_hal::digital::PinState;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub enum Service<'a> {
    //PL_SetMode,
    #[allow(non_camel_case_types)]
    PL_WakeUp,
    #[allow(non_camel_case_types)]
    PL_Transfer{data: &'a [u8], answer: &'a mut [u8]},
}

#[derive(PartialEq, Debug)]
pub enum ServiceResult<'a> {
    #[allow(non_camel_case_types)]
    PL_WakeUp,
    #[allow(non_camel_case_types)]
    PL_Transfer{answer: &'a [u8]},
}

pub enum CqOutputState {
    Disable,
    Enable,
}

pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait_us(&self, duration: u64);

    #[allow(async_fn_in_trait)]
    async fn cq_output(&self, state: CqOutputState);

    #[allow(async_fn_in_trait)]
    async fn get_cq(&self) -> PinState;

    #[allow(async_fn_in_trait)]
    async fn do_ready_pulse(&self); //TODO: maybe this needs the information whether to do the pulse up or down. Or shall it be done there? Document it!

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
                Service::PL_Transfer { data, answer } => { self.transfer(data, answer).await; }
            }
        }
    }

    pub async fn wake_up(&mut self) {
        #[allow(non_upper_case_globals)]
        const T_WU_us: u64 = 20;
        #[allow(non_upper_case_globals)]
        const T_REN_us: u64 = 500;

        self.actions.cq_output(CqOutputState::Disable).await; // TODO: is it necessary to first disable the output? To read input safely and also to prevent damage.
        self.actions.wait_us(10).await; // Typical value is 225ns.
        //let cq_state = self.actions.get_cq().await; //TODO: this is currently not taken into account and the pulse always done upwards. Improve!
        self.actions.cq_output(CqOutputState::Enable).await;
        self.actions.wait_us(10).await; // Typical value is 225ns.

        self.actions.do_ready_pulse().await;

        self.actions.cq_output(CqOutputState::Disable).await;

        self.actions.wait_us(T_REN_us - T_WU_us).await;

        RESULT_CHANNEL.send(ServiceResult::PL_WakeUp).await;
    }

    pub async fn transfer(&mut self, data: &[u8], answer: &'static mut [u8]) {
        let _result = self.actions.exchange_data(data, answer).await; //TODO: implement
        RESULT_CHANNEL.send(ServiceResult::PL_Transfer { answer }).await;
    }
}
