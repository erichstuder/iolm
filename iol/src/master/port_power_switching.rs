enum State {
    PowerOn_0,
    PowerOff_1,
}

enum Event {
    PortPowerOn,
    PortPowerOff,
    OneTimePowerOff,
}

pub trait Actions {
    // #[allow(async_fn_in_trait)] //TODO: remove
    // async fn wait_ms(&self, duration: u64);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_event(&self) -> Event;
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn confirm_event(&self, result: Result<(), EventError>);
}

pub struct StateMachine<T: Actions> {
    state: State,
    actions: T,
    off_timer_active: bool,
}

impl StateMachine {
    pub fn new(actions: Actions) -> Self{
        Self {
            state: State::PowerOn_0,
            actions,
            off_timer_active: false,
        }
    }

    pub async fn run(&self) {
        loop {
            self.next().await;
        }
    }

    async fn next() {
        match state {
            State::PowerOn_0 => {
                match self.state_actions.await_event().await {
                    Event::PortPowerOn => {
                        self.actions.confirm_event(Ok(())).await;
                    }
                    Event::PortPowerOff => {
                        self.actions.confirm_event(Ok(())).await;
                        //TODO: switch port power off
                        self.state = State::PortPowerOff;
                    }
                    Event::OneTimePowerOff => {
                        self.actions.confirm_event(Ok(())).await;
                        //TODO: switch port power off and start OffTimer with PowerOffTime
                        self.off_timer_active = true;
                        self.state = State::PortPowerOff;
                    }
                }
            },
            State::PowerOff_1 => {
                // TODO: wie wird das mit dem Timeout gelöst?
                // siehe auch T7 Guard
                let event = match self.off_timer_active {
                    false => self.state_actions.await_event().await,
                    true => self.state_actions.await_event_with_timeout().await,
                };

                match Some(event) {
                    Event::PortPowerOff => {
                        self.actions.confirm_event(Ok(())).await;
                        self.off_timer_active = false;
                    }
                    Event::OneTimePowerOff => {
                        self.actions.confirm_event(Ok(())).await;
                        self.off_timer_active = true;
                    }
                    Event::PortPowerOn => {
                        self.actions.confirm_event(Ok(())).await;
                        //TODO: switch port power on
                        self.off_timer_active = false;
                        self.state = State::PowerOn_0;
                    }
                }
            }
        }
    }
}
