// use defmt::info;

mod master_dl_mode_handler;

use core::marker::PhantomData;

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

pub struct DL<A: Actions, T: StateActions> {
    // m_sequence_time: MSequenceTime,
    // m_sequence_type: MSequenceType,
    // pd_input_length: PDInputLength,
    // pd_output_length: PDOutputLength,
    // on_req_data_length_per_message: OnReqDataLengthPerMessage,

    //dl_mode_handler_state_machine: master_dl_mode_handler::StateMachine<T>,
    actions: A,
    dummy: PhantomData<T>,
}

impl <A: Actions, T: StateActions> DL<A, T> {

    pub fn new(actions: A, state_actions: T) -> (Self, master_dl_mode_handler::StateMachine<T>) {
        (Self{
            //dl_mode_handler_state_machine: master_dl_mode_handler::StateMachine::new(state_actions),
            actions,
            dummy: PhantomData,
        },
            master_dl_mode_handler::StateMachine::new(state_actions),
        )
    }

    // pub async fn run(&mut self) {
    //     self.dl_mode_handler_state_machine.run().await;
    // }

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

        self.actions.send_dl_mode_handler_event(event).await;
        self.actions.await_dl_mode_handler_event_confirmation().await;

        // if self.dl_mode_handler_state_machine.parse_event(event).await.is_ok() {
        //     Ok(())
        // }
        // else {
        //     Err(ErrorInfo::STATE_CONFLICT)
        // }

        Ok(())
    }
}
