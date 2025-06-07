// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

pub mod dl_mode_handler;
pub type DlModeHandlerStateMachine<T> = dl_mode_handler::StateMachine<DlModeHandlerActionsImpl<T>>;
pub use dl_mode_handler::ReadyPulseResult as ReadyPulseResult;


pub enum Mode {
    #[allow(unused)] //TODO: remove
    INACTIVE,
    STARTUP,
    #[allow(unused)] //TODO: remove
    PREOPERATE,
    #[allow(unused)] //TODO: remove
    OPERATE,
}

#[allow(unused)] //TODO: remove
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
    #[allow(unused)] //TODO: remove
    STATE_CONFLICT,
    #[allow(non_camel_case_types)]
    #[allow(unused)] //TODO: remove
    PARAMETER_CONFLICT,
}

pub trait Actions {
    #[allow(async_fn_in_trait)] //TODO: remove
    async fn wait_ms(&self, duration: u64);

    #[allow(async_fn_in_trait)] //TODO: remove
    async fn port_power_off_on_ms(&self, duration: u64);

    #[allow(async_fn_in_trait)] //TODO: remove
    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> ReadyPulseResult;
}

pub struct DlModeHandlerActionsImpl<T: Actions>{
    pub actions: T,
}

impl<T: Actions> dl_mode_handler::Actions for DlModeHandlerActionsImpl<T> {
    async fn wait_ms(&self, duration: u64) {
        self.actions.wait_ms(duration).await;
    }

    async fn port_power_off_on_ms(&self, duration: u64) {
        self.actions.port_power_off_on_ms(duration).await;
    }

    async fn await_ready_pulse_with_timeout_ms(&self, duration: u64) -> ReadyPulseResult {
        self.actions.await_ready_pulse_with_timeout_ms(duration).await
    }
}

pub struct DL<T: Actions> {
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
            dl_mode_handler::StateMachine::new(
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
            Mode::INACTIVE => dl_mode_handler::Event::DL_SetMode_INACTIVE,
            Mode::STARTUP => dl_mode_handler::Event::DL_SetMode_STARTUP,
            Mode::PREOPERATE => dl_mode_handler::Event::DL_SetMODE_PREOPERATE,
            Mode::OPERATE => dl_mode_handler::Event::DL_SetMODE_OPERATE,
        };

        dl_mode_handler::EVENT_CHANNEL.send(event).await;
         // At the moment we just panic here on error. I don't know how to handle this error yet.
        dl_mode_handler::RESULT_CHANNEL.receive().await.unwrap();
        Ok(())
    }
}
