mod master_dl_mode_handler;

pub use master_dl_mode_handler::StateActions;

pub struct DL<T: StateActions> {
    // dl_mode_handler: dl_mode_handler::master::
    // #[cfg(feature = "master")]
    // m_sequence_time: MSequenceTime,
    // #[cfg(feature = "master")]
    // m_sequence_type: MSequenceType,
    // #[cfg(feature = "master")]
    // pd_input_length: PDInputLength,
    // #[cfg(feature = "master")]
    // pd_output_length: PDOutputLength,
    // #[cfg(feature = "master")]
    // on_req_data_length_per_message: OnReqDataLengthPerMessage,


    dl_mode_handler_state_machine: master_dl_mode_handler::StateMachine<T>,

    #[allow(non_snake_case)]
    pub DL_SetMode: dl_set_mode::DL_SetMode,
}

impl <T: StateActions> DL<T> {
    pub fn new(state_actions: T) -> Self {
        Self{
            dl_mode_handler_state_machine: master_dl_mode_handler::StateMachine::new(state_actions),
            DL_SetMode: dl_set_mode::DL_SetMode,
        }
    }
}

pub mod dl_set_mode {
    #[allow(non_camel_case_types)]
    pub struct DL_SetMode;
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

    pub enum ErrorInfo {
        #[allow(non_camel_case_types)]
        STATE_CONFLICT,
        #[allow(non_camel_case_types)]
        PARAMETER_CONFLICT,
    }

    impl DL_SetMode {
        pub fn call(&self, _mode: Mode, _value_list: ValueList) -> Result<(), ErrorInfo> {
            // dl.m_sequence_time = value_list.m_sequence_time;
            // dl.m_sequence_type = value_list.m_sequence_type;
            // dl.pd_input_length = value_list.pd_input_length;
            // dl.pd_output_length = value_list.pd_output_length;
            // dl.on_req_data_length_per_message = value_list.on_req_data_length_per_message;

            // TODO: send state change request to statemachine and answer with Result
            Ok(())
        }
    }
}
