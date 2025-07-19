// #[cfg(feature = "log")]
// use log::info;
// #[cfg(feature = "defmt")]
// use defmt::info;

use futures;
use core::time::Duration;

mod dl_services;
pub use dl_services::outside_dl as services;
pub use dl_services::Service;
pub use dl_services::{ dl_set_mode, dl_mode };

mod dl_mode_handler;
pub type DlModeHandlerStateMachine<A> = dl_mode_handler::StateMachine<DlModeHandlerActionsImpl<A>>;
pub use dl_mode_handler::ReadyPulseResult as ReadyPulseResult;

mod message_handler;


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
    #[allow(async_fn_in_trait)]
    async fn wait(&self, duration: Duration);

    #[allow(async_fn_in_trait)]
    async fn port_power_off_on(&self, duration: Duration);

    #[allow(async_fn_in_trait)]
    async fn await_ready_pulse_with_timeout(&self, duration: Duration) -> ReadyPulseResult;
}

pub struct DlModeHandlerActionsImpl<A: Actions>{
    pub actions: A,
}

impl<A: Actions> dl_mode_handler::Actions for DlModeHandlerActionsImpl<A> {
    async fn wait(&self, duration: Duration) {
        self.actions.wait(duration).await;
    }

    async fn port_power_off_on(&self, duration: Duration) {
        self.actions.port_power_off_on(duration).await;
    }

    async fn await_ready_pulse_with_timeout(&self, duration: Duration) -> ReadyPulseResult {
        self.actions.await_ready_pulse_with_timeout(duration).await
    }
}

pub struct DL<A:Actions> {
    // m_sequence_time: MSequenceTime,
    // m_sequence_type: MSequenceType,
    // pd_input_length: PDInputLength,
    // pd_output_length: PDOutputLength,
    // on_req_data_length_per_message: OnReqDataLengthPerMessage,

    _actions: A, //unused at the moment, maybe later
    dl_mode_handler: dl_mode_handler::StateMachine<DlModeHandlerActionsImpl<A>>,
    message_handler: message_handler::MessageHandler,
}

impl<A: Actions + Copy> DL<A> {
    pub fn new(actions: A) -> Self {
        Self {
            _actions: actions,
            dl_mode_handler: dl_mode_handler::StateMachine::new(DlModeHandlerActionsImpl{ actions }),
            message_handler: message_handler::MessageHandler::new(),
        }
    }

    pub async fn run(&mut self) {
        futures::join!(
            self.dl_mode_handler.run(),
            self.message_handler.run(),
        );
    }

    // #[allow(non_snake_case)]
    // pub async fn DL_SetMode(mode: Mode/*, _value_list: ValueList*/) -> Result<(), ErrorInfo> {
    //     // self.m_sequence_time = value_list.m_sequence_time;
    //     // self.m_sequence_type = value_list.m_sequence_type;
    //     // self.pd_input_length = value_list.pd_input_length;
    //     // self.pd_output_length = value_list.pd_output_length;
    //     // self.on_req_data_length_per_message = value_list.on_req_data_length_per_message;

    //     let event = match mode {
    //         Mode::INACTIVE => dl_mode_handler::Service::DL_SetMode_INACTIVE,
    //         Mode::STARTUP => dl_mode_handler::Service::DL_SetMode_STARTUP,
    //         Mode::PREOPERATE => dl_mode_handler::Service::DL_SetMODE_PREOPERATE,
    //         Mode::OPERATE => dl_mode_handler::Service::DL_SetMODE_OPERATE,
    //     };

    //     dl_mode_handler::SERVICE_REQ.send(event).await;
    //      // At the moment we just panic here on error. I don't know how to handle this error yet.
    //     dl_mode_handler::SERVICE_CNF.receive().await.unwrap();
    //     Ok(())
    // }
}
