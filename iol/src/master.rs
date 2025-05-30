// see #11

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod port_power_switching;
pub type PortPowerSwitchingStateMachine<T> = port_power_switching::StateMachine<PortPowerSwitchingActionsImpl<T>>;

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_on(&self);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_off(&self);

    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_with_timeout_ms<F, T>(&self, future: F, duration: u64) -> Option<T>
    where
        F: core::future::Future<Output = T> + Send;
}

static PORT_POWER_SWITCHING_EVENT_CHANNEL: Channel<CriticalSectionRawMutex, port_power_switching::Event, 1> = Channel::new();
static PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

pub struct PortPowerSwitchingActionsImpl<T: Actions> {
    actions: T
}

impl<T: Actions> port_power_switching::Actions for PortPowerSwitchingActionsImpl<T> {
    async fn port_power_on(&self) {
        self.actions.port_power_on().await;
    }

    async fn port_power_off(&self) {
        self.actions.port_power_off().await;
    }

    async fn await_event(&self) -> port_power_switching::Event {
        PORT_POWER_SWITCHING_EVENT_CHANNEL.receive().await
    }

    async fn await_event_with_timeout_ms(&self, duration: u64) -> port_power_switching::Event {
        match self.actions.await_with_timeout_ms(
            PORT_POWER_SWITCHING_EVENT_CHANNEL.receive(),
            duration,
        ).await {
            Some(event) => event,
            None => port_power_switching::Event::OffTimerElapsed,
        }
    }

    async fn confirm_event(&self) {
        PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.send(()).await
    }
}

pub struct Master<T: Actions> {
    _actions: T, //unused at the moment. maybe later.
}

impl<T: Actions + Copy> Master<T> {
    pub fn new(actions: T) -> (Self, PortPowerSwitchingStateMachine<T>) {
        (
            Self {
                _actions: actions,
            },
            port_power_switching::StateMachine::new(
                PortPowerSwitchingActionsImpl {
                    actions,
                }
            )
        )
    }

    //Some helper functions for the moment. They may be removed in the future.
    pub async fn port_power_on(&self) {
        PORT_POWER_SWITCHING_EVENT_CHANNEL.send(port_power_switching::Event::PortPowerOn).await;
        PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.receive().await;
    }

    pub async fn port_power_off(&self) {
        PORT_POWER_SWITCHING_EVENT_CHANNEL.send(port_power_switching::Event::PortPowerOff).await;
        PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.receive().await;
    }

    pub async fn port_power_off_on(&self, duration: u64) {
        PORT_POWER_SWITCHING_EVENT_CHANNEL.send(port_power_switching::Event::OneTimePowerOff(duration)).await;
        PORT_POWER_SWITCHING_EVENT_RESULT_CHANNEL.receive().await;
    }
}
