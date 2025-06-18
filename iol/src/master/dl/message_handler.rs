mod state_machine;
pub use state_machine::EVENT_CHANNEL;
pub use state_machine::RESULT_CHANNEL;
pub use state_machine::Event;
pub use state_machine::TransmissionRate;

mod m_sequences;

pub struct MessageHandler {
    state_machine: state_machine::StateMachine,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            state_machine: state_machine::StateMachine::new(),
        }
    }

    pub async fn run(&mut self) {
        self.state_machine.run().await;
    }
}
