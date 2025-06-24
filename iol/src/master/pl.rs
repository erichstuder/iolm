// see #5.3.3.3

// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

pub use embedded_hal::digital::PinState;

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

    #[allow(non_snake_case)]
    pub async fn PL_WakeUp(&mut self) {
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
    }
}
