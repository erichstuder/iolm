#[cfg(feature = "log")]
use log::info;
#[cfg(feature = "defmt")]
use defmt::info;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

mod master_dl_mode_handler;
pub type DlModeHandlerStateMachine<T> = master_dl_mode_handler::StateMachine<DlModeHandlerActionsImpl<T>>;


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
    async fn wait_ms(&self, duration: u64);
}

static DL_MODE_HANDLER_EVENT_CHANNEL: Channel<CriticalSectionRawMutex, master_dl_mode_handler::Event, 1> = Channel::new();
static DL_MODE_HANDLER_EVENT_RESULT_CHANNEL: Channel<CriticalSectionRawMutex, Result<(), master_dl_mode_handler::EventError>, 1> = Channel::new();

pub struct DlModeHandlerActionsImpl<T: Actions>{
    pub actions: T,
}

impl<T: Actions> master_dl_mode_handler::Actions for DlModeHandlerActionsImpl<T> {
    async fn wait_ms(&self, duration: u64) {
        self.actions.wait_ms(duration).await;
    }

    async fn await_event(&self) -> master_dl_mode_handler::Event {
        DL_MODE_HANDLER_EVENT_CHANNEL.receive().await
    }

    async fn confirm_event(&self, result: Result<(), master_dl_mode_handler::EventError>) {
        DL_MODE_HANDLER_EVENT_RESULT_CHANNEL.send(result).await;
        info!("signalled");
    }
}

pub struct DL<T> {
    // m_sequence_time: MSequenceTime,
    // m_sequence_type: MSequenceType,
    // pd_input_length: PDInputLength,
    // pd_output_length: PDOutputLength,
    // on_req_data_length_per_message: OnReqDataLengthPerMessage,

    _actions: T //unused at the moment, maybe later
}

impl<T: Actions + Copy> DL<T> {
    pub fn new(actions: T) -> (Self, DlModeHandlerStateMachine<T>) {
        (
            Self{
                _actions: actions,
            },
            master_dl_mode_handler::StateMachine::new(
                DlModeHandlerActionsImpl{
                    actions,
                }
            ),
        )
    }

    #[allow(non_snake_case)]
    pub async fn DL_SetMode(&mut self, mode: Mode/*, _value_list: ValueList*/) -> Result<(), ErrorInfo> {
        // self.m_sequence_time = value_list.m_sequence_time;
        // self.m_sequence_type = value_list.m_sequence_type;
        // self.pd_input_length = value_list.pd_input_length;
        // self.pd_output_length = value_list.pd_output_length;
        // self.on_req_data_length_per_message = value_list.on_req_data_length_per_message;

        let event = match mode {
            Mode::INACTIVE => master_dl_mode_handler::Event::DL_SetMode_INACTIVE,
            Mode::STARTUP => master_dl_mode_handler::Event::DL_SetMode_STARTUP,
            Mode::PREOPERATE => master_dl_mode_handler::Event::DL_SetMODE_PREOPERATE,
            Mode::OPERATE => master_dl_mode_handler::Event::DL_SetMODE_OPERATE,
        };

        DL_MODE_HANDLER_EVENT_CHANNEL.send(event).await;
         // At the moment we just panic here on error. I don't know how to handle an error yet.
        DL_MODE_HANDLER_EVENT_RESULT_CHANNEL.receive().await.unwrap();
        Ok(())
    }
}
