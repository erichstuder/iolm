#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_time::Timer;

mod master_dl_mode_handler;

// TODO: These pub use should not be done like this. It is then no longer clear where they belong to.
pub use master_dl_mode_handler::StateActions;
pub use master_dl_mode_handler::StateMachine;
pub use master_dl_mode_handler::Event;
pub use master_dl_mode_handler::EventError;

pub enum Mode {
    INACTIVE,
    STARTUP,
    PREOPERATE,
    OPERATE,
}

pub struct ValueList {
    // m_sequence_time: MSequenceTime,
    // m_sequence_type: MSequenceType,
    // pd_input_length: PDInputLength,
    // pd_output_length: PDOutputLength,
    // on_req_data_length_per_message: OnReqDataLengthPerMessage,
}

#[derive(Debug)]
pub enum ErrorInfo {
    #[allow(non_camel_case_types)]
    STATE_CONFLICT,
    #[allow(non_camel_case_types)]
    PARAMETER_CONFLICT,
}

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn send_dl_mode_handler_event(&self, event: master_dl_mode_handler::Event);
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_dl_mode_handler_event_confirmation(&self);
}

static DL_MODE_HANDLER_EVENT_CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();
static DL_MODE_HANDLER_EVENT_RESULT_CHANNEL: Channel<CriticalSectionRawMutex, Result<(), EventError>, 1> = Channel::new();

// TODO: This should not be public struct
pub struct StateActionsImpl;
impl StateActions for StateActionsImpl {
    async fn wait_ms(&self, duration: u64) {
        // TODO: implement
        // Timer::after_millis(duration).await;
    }

    async fn await_event(&self) -> Event {
        DL_MODE_HANDLER_EVENT_CHANNEL.receive().await
    }

    async fn confirm_event(&self, result: Result<(), EventError>) {
        DL_MODE_HANDLER_EVENT_RESULT_CHANNEL.send(result).await;
        info!("signalled");
    }
}

pub struct DL {
    // m_sequence_time: MSequenceTime,
    // m_sequence_type: MSequenceType,
    // pd_input_length: PDInputLength,
    // pd_output_length: PDOutputLength,
    // on_req_data_length_per_message: OnReqDataLengthPerMessage,
}

impl DL {

    pub fn new() -> (Self, master_dl_mode_handler::StateMachine<StateActionsImpl>) {
        (
            Self{},
            master_dl_mode_handler::StateMachine::new(StateActionsImpl),
        )
    }

    #[allow(non_snake_case)]
    pub async fn DL_SetMode(&mut self, mode: Mode/*, _value_list: ValueList*/) -> Result<(), ErrorInfo> {
        // self.m_sequence_time = value_list.m_sequence_time;
        // self.m_sequence_type = value_list.m_sequence_type;
        // self.pd_input_length = value_list.pd_input_length;
        // self.pd_output_length = value_list.pd_output_length;
        // self.on_req_data_length_per_message = value_list.on_req_data_length_per_message;

        // TODO: send state change request to statemachine and answer with Result => I dont like it => Remove
        let event = match mode {
            Mode::INACTIVE => master_dl_mode_handler::Event::DL_SetMode_INACTIVE,
            Mode::STARTUP => master_dl_mode_handler::Event::DL_SetMode_STARTUP,

            //TODO: add more
            _ => master_dl_mode_handler::Event::DL_SetMode_INACTIVE,
        };

        // self.actions.send_dl_mode_handler_event(event).await;
        // self.actions.await_dl_mode_handler_event_confirmation().await;
        DL_MODE_HANDLER_EVENT_CHANNEL.send(event).await;
        DL_MODE_HANDLER_EVENT_RESULT_CHANNEL.receive().await.unwrap();

        // if self.dl_mode_handler_state_machine.parse_event(event).await.is_ok() {
        //     Ok(())
        // }
        // else {
        //     Err(ErrorInfo::STATE_CONFLICT)
        // }

        Ok(())
    }
}
