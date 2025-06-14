// see #11

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

mod port_power_switching;
pub type PortPowerSwitchingStateMachine<A> = port_power_switching::StateMachine<PortPowerSwitchingActions<A>>;

mod pl;
use pl::PL;
pub use pl::CqOutputState as CqOutputState;
pub use pl::PinState as PinState;

mod dl;
use dl::DL;
pub use dl::ReadyPulseResult as ReadyPulseResult;
pub type DlModeHandlerStateMachine<A> = dl::DlModeHandlerStateMachine<DlActions<A>, PlActions<A>>;

pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn wait_us(&self, duration: u64);

    #[allow(async_fn_in_trait)]
    async fn wait_ms(&self, duration: u64);

    #[allow(async_fn_in_trait)]
    async fn cq_output(&self, state: CqOutputState);

    #[allow(async_fn_in_trait)]
    async fn get_cq(&self) -> PinState;

    #[allow(async_fn_in_trait)]
    async fn do_ready_pulse(&self);

    #[allow(async_fn_in_trait)]
    async fn port_power_on(&self);

    #[allow(async_fn_in_trait)]
    async fn port_power_off(&self);

    #[allow(async_fn_in_trait)]
    async fn await_event_with_timeout_ms<F, T>(&self, duration: u64, future: F) -> Option<T>
    where
        F: core::future::Future<Output = T> + Send;

    #[allow(async_fn_in_trait)]
    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> ReadyPulseResult;

    #[allow(async_fn_in_trait)]
    async fn exchange_m_sequence();
}

pub struct PlActions<A: Actions> {
    actions: A,
}

impl<A: Actions> pl::Actions for PlActions<A> {
    async fn wait_us(&self, duration: u64) {
        self.actions.wait_us(duration).await;
    }

    async fn cq_output(&self, state: CqOutputState) {
        self.actions.cq_output(state).await;
    }

    async fn get_cq(&self) -> PinState {
        self.actions.get_cq().await
    }

    async fn do_ready_pulse(&self) {
        self.actions.do_ready_pulse().await
    }

    fn exchange_data(&self) {
        //TODO: implementation
    }
}

pub struct PortPowerSwitchingActions<A: Actions> {
    actions: A,
}

impl<A: Actions> port_power_switching::Actions for PortPowerSwitchingActions<A> {
    async fn port_power_on(&self) {
        self.actions.port_power_on().await;
    }

    async fn port_power_off(&self) {
        self.actions.port_power_off().await;
    }

    async fn await_event_with_timeout_ms(&self, duration: u64) -> port_power_switching::Event {
        match self.actions.await_event_with_timeout_ms(duration, port_power_switching::EVENT_CHANNEL.receive()).await {
            Some(event) => event,
            None => port_power_switching::Event::OffTimerElapsed,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DlActions<A: Actions> {
    actions: A,
}

impl<A: Actions> dl::Actions for DlActions<A> {
    async fn wait_ms(&self, duration: u64) {
        self.actions.wait_ms(duration).await;
    }

    async fn port_power_off_on_ms(&self, duration: u64) {
        info!("port power off on");
        port_power_switching::EVENT_CHANNEL.send(port_power_switching::Event::OneTimePowerOff(duration)).await;
        port_power_switching::RESULT_CHANNEL.receive().await;
        info!("port power off on: done");
    }

    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> ReadyPulseResult {
        self.actions.await_ready_pulse_with_timeout_ms(duration).await
    }
}

pub struct Master<A: Actions> {
    _actions: A, //unused at the moment. maybe later.
    dl: DL<DlActions<A>, PlActions<A>>,
}

impl<A: Actions + Copy> Master<A> {
    pub fn new(actions: A) -> (Self, PortPowerSwitchingStateMachine<A>, DlModeHandlerStateMachine<A>) {
        let port_power_switching_state_machine = port_power_switching::StateMachine::new(
                PortPowerSwitchingActions { actions }
            );

        let pl = PL::new(PlActions { actions });

        let (dl, dl_mode_handler_state_machine) = DL::new(DlActions { actions }, pl);

        (
            Self {
                _actions: actions,
                dl,
            },
            port_power_switching_state_machine,
            dl_mode_handler_state_machine,
        )
    }

    //Some helper functions for the moment. They may be removed in the future.

    // pub async fn port_power_on(&self) {
    //     PORT_POWER_SWITCHING_EVENT_CHANNEL.send(port_power_switching::Event::PortPowerOn).await;
    //     PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.receive().await;
    // }

    // pub async fn port_power_off(&self) {
    //     PORT_POWER_SWITCHING_EVENT_CHANNEL.send(port_power_switching::Event::PortPowerOff).await;
    //     PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.receive().await;
    // }

    // pub async fn port_power_off_on(&self, duration: u64) {
    //     PORT_POWER_SWITCHING_EVENT_CHANNEL.send(port_power_switching::Event::OneTimePowerOff(duration)).await;
    //     PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.receive().await;
    // }

    pub async fn dl_set_mode_startup(&mut self) {
        self.dl.DL_SetMode(dl::Mode::STARTUP).await.unwrap();
    }
}
