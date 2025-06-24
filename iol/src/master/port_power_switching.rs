// see #11.8

#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

#[cfg(test)]
use mockall::automock;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[derive(Debug, PartialEq)]
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

#[cfg_attr(test, automock)]
pub trait Actions {
    #[allow(async_fn_in_trait)]
    async fn port_power_on(&self);

    #[allow(async_fn_in_trait)]
    async fn port_power_off(&self);

    #[allow(async_fn_in_trait)]
    async fn await_event_with_timeout_ms(&self, duration: u64) -> Event;
}

pub static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();
pub static RESULT_CHANNEL: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

pub struct StateMachine<A: Actions> {
    state: State,
    actions: A,
    off_timer_active: bool,
    off_time: u64,
}

impl<A: Actions> StateMachine<A> {
    pub fn new(actions: A) -> Self{
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
                        // Note: No event confirmation here. We confirm as soon as the OneTimerPowerOff is finished.
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


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOn_0_state_PortPowerOn_event() {
        let mut sm = StateMachine::new(MockActions::new());
        assert_eq!(sm.state, State::PowerOn_0);
        EVENT_CHANNEL.send(Event::PortPowerOn).await;
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOn_0);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOn_0_state_PortPowerOff_event() {
        let mut sm = StateMachine::new(MockActions::new());
        assert_eq!(sm.state, State::PowerOn_0);
        EVENT_CHANNEL.send(Event::PortPowerOff).await;
        sm.actions.expect_port_power_off()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOn_0_state_OneTimerPowerOff_event() {
        let mut sm = StateMachine::new(MockActions::new());
        assert_eq!(sm.state, State::PowerOn_0);
        EVENT_CHANNEL.send(Event::OneTimePowerOff(0)).await;
        sm.actions.expect_port_power_off()
            .times(1)
            .returning(|| ());
        sm.next().await;
        // assert_eq!(RESULT_CHANNEL.receive().await, ()); No immediate confirmation
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn panic_in_PowerOn_0() {
        let mut sm = StateMachine::new(MockActions::new());
        assert_eq!(sm.state, State::PowerOn_0);

        EVENT_CHANNEL.send(Event::OffTimerElapsed).await;
        let handle = tokio::spawn(async move {
            sm.next().await;
        });
        let result = handle.await;
        assert!(result.is_err());
    }

    #[allow(non_snake_case)]
    async fn go_to_PowerOff_1(sm: &mut StateMachine<MockActions>) {
        EVENT_CHANNEL.send(Event::PortPowerOff).await;
        sm.actions.expect_port_power_off()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOff_1_state_PortPowerOff_event() {
        let mut sm = StateMachine::new(MockActions::new());
        go_to_PowerOff_1(&mut sm).await;
        EVENT_CHANNEL.send(Event::PortPowerOff).await;
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOff_1_state_OneTimePowerOff_event() {
        let mut sm = StateMachine::new(MockActions::new());
        go_to_PowerOff_1(&mut sm).await;
        EVENT_CHANNEL.send(Event::OneTimePowerOff(42)).await;
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOff_1_state_PowerOn_event() {
        let mut sm = StateMachine::new(MockActions::new());
        go_to_PowerOff_1(&mut sm).await;
        EVENT_CHANNEL.send(Event::PortPowerOn).await;
        sm.actions.expect_port_power_on()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOn_0);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn PowerOff_1_state_OffTimerElapsed_event() {
        let mut sm = StateMachine::new(MockActions::new());
        go_to_PowerOff_1(&mut sm).await;
        EVENT_CHANNEL.send(Event::OffTimerElapsed).await;
        sm.actions.expect_port_power_on()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.state, State::PowerOn_0);
    }

    #[allow(non_snake_case)]
    async fn start_OneTimePowerOff() -> (StateMachine<MockActions>, u64) {
        let mut sm = StateMachine::new(MockActions::new());
        assert_eq!(sm.off_timer_active, false);
        assert_eq!(sm.off_time, 0);

        let off_time = 222 as u64;
        EVENT_CHANNEL.send(Event::OneTimePowerOff(off_time)).await;
        sm.actions.expect_port_power_off()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(sm.off_timer_active, true);
        assert_eq!(sm.off_time, off_time);
        assert_eq!(sm.state, State::PowerOff_1);
        (sm, off_time)

    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn OneTimePowerOff_aborted_with_PortPowerOff() {
        let (mut sm, off_time) = start_OneTimePowerOff().await;

        sm.actions.expect_await_event_with_timeout_ms()
            .times(1)
            .withf(move |duration| *duration == off_time)
            .returning(|_| Event::PortPowerOff);
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.off_timer_active, false);
        assert_eq!(sm.off_time, off_time);
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn OneTimePowerOff_aborted_with_OneTimePowerOff() {
        let (mut sm, off_time) = start_OneTimePowerOff().await;

        let off_time_2 = 16 as u64;
        sm.actions.expect_await_event_with_timeout_ms()
            .times(1)
            .withf(move |duration| *duration == off_time)
            .returning(move |_| Event::OneTimePowerOff(off_time_2));
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.off_timer_active, true);
        assert_eq!(sm.off_time, off_time_2);
        assert_eq!(sm.state, State::PowerOff_1);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn OneTimePowerOff_aborted_with_PortPowerOn() {
        let (mut sm, off_time) = start_OneTimePowerOff().await;

        sm.actions.expect_await_event_with_timeout_ms()
            .times(1)
            .withf(move |duration| *duration == off_time)
            .returning(|_| Event::PortPowerOn);
        sm.actions.expect_port_power_on()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.off_timer_active, false);
        assert_eq!(sm.off_time, off_time);
        assert_eq!(sm.state, State::PowerOn_0);
    }

    #[tokio::test]
    #[allow(non_snake_case)]
    async fn OneTimePowerOff_success() {
        let (mut sm, off_time) = start_OneTimePowerOff().await;

        sm.actions.expect_await_event_with_timeout_ms()
            .times(1)
            .withf(move |duration| *duration == off_time)
            .returning(|_| Event::OffTimerElapsed);
        sm.actions.expect_port_power_on()
            .times(1)
            .returning(|| ());
        sm.next().await;
        assert_eq!(RESULT_CHANNEL.receive().await, ());
        assert_eq!(sm.off_timer_active, false);
        assert_eq!(sm.off_time, off_time);
        assert_eq!(sm.state, State::PowerOn_0);
    }
}
