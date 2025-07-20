mod state_machine;
pub use state_machine::EVENT_CHANNEL;
pub use state_machine::RESULT_CHANNEL;
pub use state_machine::{Event, Actions};

mod m_sequences;

pub struct MessageHandler<A> {
    state_machine: state_machine::StateMachine<A>,
}

impl<A: Actions> MessageHandler<A> {
    pub fn new(actions: A) -> Self {
        Self {
            state_machine: state_machine::StateMachine::new(actions),
        }
    }

    pub async fn run(&mut self) {
        self.state_machine.run().await;
    }
}
