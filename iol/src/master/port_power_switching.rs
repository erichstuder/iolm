// see #11.8

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub enum State {
    #[allow(non_camel_case_types)]
    PowerOn_0,
    #[allow(non_camel_case_types)]
    PowerOff_1,
}

pub enum Event {
    PortPowerOn,
    PortPowerOff,
    OneTimePowerOff(u64),
    // Note: It is more elegant if OffTimerElapsed is also an Event instead of a Guard.
    OffTimerElapsed,
}

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_on(&self);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_off(&self);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_event_with_timeout_ms(&self, duration: u64) -> Event;
}

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

pub struct StateMachine<T: Actions> {
    state: State,
    actions: T,
    off_timer_active: bool,
    off_time: u64,
}

impl<T: Actions> StateMachine<T> {
    pub fn new(actions: T) -> Self{
        Self {
            state: State::PowerOn_0,
            actions,
            off_timer_active: false,
            off_time: 0,
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.next().await;
        }
    }

    async fn await_event(&self) -> Event {
        EVENT_CHANNEL.receive().await
    }

    async fn confirm_event(&self) {
        RESULT_CHANNEL.send(()).await;
    }

    async fn next(&mut self) {
        match self.state {
            State::PowerOn_0 => {
                info!("entering PowerOn_0");
                match self.await_event().await {
                    Event::PortPowerOn => {
                        self.confirm_event().await;
                    }
                    Event::PortPowerOff => {
                        self.actions.port_power_off().await;
                        self.state = State::PowerOff_1;
                        self.confirm_event().await;
                    }
                    Event::OneTimePowerOff(duration) => {
                        self.actions.port_power_off().await;
                        self.off_timer_active = true;
                        self.off_time = duration;
                        self.state = State::PowerOff_1;
                        self.confirm_event().await;
                    }
                    _ => panic!("This should never ever happen!")
                }
            },
            State::PowerOff_1 => {
                info!("entering PowerOff_1");
                let event = match self.off_timer_active {
                    false => self.await_event().await,
                    true => self.actions.await_event_with_timeout_ms(self.off_time).await,
                };

                match event {
                    Event::PortPowerOff => {
                        self.off_timer_active = false;
                        self.confirm_event().await;
                    }
                    Event::OneTimePowerOff(duration) => {
                        self.off_timer_active = true;
                        self.off_time = duration;
                        self.confirm_event().await;
                    }
                    Event::PortPowerOn | Event::OffTimerElapsed  => {
                        self.actions.port_power_on().await;
                        self.off_timer_active = false;
                        self.state = State::PowerOn_0;
                        self.confirm_event().await;
                    }
                }
            }
        }
    }
}
