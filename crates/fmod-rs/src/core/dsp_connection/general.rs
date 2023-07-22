use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # General.
impl DspConnection {
    /// Retrieves the connection's input DSP unit.
    ///
    /// If [`Dsp::add_input`] was just called, the connection might not be ready
    /// because the DSP system is still queued to be connected, and may need to
    /// wait several milliseconds for the next mix to occur. If so the function
    /// will return [`Error::NotReady`].
    pub fn get_input(&self) -> Result<&Dsp> {
        let mut dsp = ptr::null_mut();
        ffi!(FMOD_DSPConnection_GetInput(self.as_raw(), &mut dsp))?;
        Ok(unsafe { Dsp::from_raw(dsp) })
    }

    /// Retrieves the connection's output DSP unit.
    ///
    /// If [`Dsp::add_input`] was just called, the connection might not be ready
    /// because the DSP system is still queued to be connected, and may need to
    /// wait several milliseconds for the next mix to occur. If so the function
    /// will return [`Error::NotReady`].
    pub fn get_output(&self) -> Result<&Dsp> {
        let mut dsp = ptr::null_mut();
        ffi!(FMOD_DSPConnection_GetOutput(self.as_raw(), &mut dsp))?;
        Ok(unsafe { Dsp::from_raw(dsp) })
    }

    /// Retrieves the type of the connection between 2 DSP units.
    pub fn get_type(&self) -> Result<DspConnectionType> {
        let mut kind = DspConnectionType::zeroed();
        ffi!(FMOD_DSPConnection_GetType(self.as_raw(), kind.as_raw_mut()))?;
        Ok(kind)
    }

    // set_user_data, get_user_data
}

fmod_enum! {
    /// List of connection types between 2 DSP nodes.
    #[derive(Default)]
    pub enum DspConnectionType: FMOD_DSPCONNECTION_TYPE {
        #[default]
        /// Default connection type. Audio is mixed from the input to the output [Dsp]'s audible buffer.
        ///
        /// Default [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s audible buffer, meaning it will be part of the audible signal. A standard connection will execute its input [Dsp] if it has not been executed before.
        Standard      = FMOD_DSPCONNECTION_TYPE_STANDARD,
        /// Sidechain connection type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer.
        ///
        /// Sidechain [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, meaning it will NOT be part of the audible signal. A sidechain connection will execute its input [Dsp] if it has not been executed before.
        ///
        /// The purpose of the seperate sidechain buffer in a [Dsp], is so that the [Dsp] effect can privately access for analysis purposes. An example of use in this case, could be a compressor which analyzes the signal, to control its own effect parameters (ie a compression level or gain).
        ///
        /// For the effect developer, to accept sidechain data, the sidechain data will appear in the [DspState] struct which is passed into the read callback of a [Dsp] unit.
        ///
        /// [DspState::sidechain_data] and [DspState::sidechain_channels] will hold the mixed result of any sidechain data flowing into it.
        Sidechain     = FMOD_DSPCONNECTION_TYPE_SIDECHAIN,
        /// Send connection type. Audio is mixed from the input to the output [Dsp]'s audible buffer, but the input is NOT executed, only copied from. A standard connection or sidechain needs to make an input execute to generate data.
        ///
        /// Send [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s audible buffer, meaning it will be part of the audible signal. A send connection will NOT execute its input [Dsp] if it has not been executed before.
        ///
        /// A send connection will only read what exists at the input's buffer at the time of executing the output [Dsp] unit (which can be considered the 'return')
        Send          = FMOD_DSPCONNECTION_TYPE_SEND,
        /// Send sidechain connection type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, but the input is NOT executed, only copied from. A standard connection or sidechain needs to make an input execute to generate data.
        ///
        /// Send sidechain [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, meaning it will NOT be part of the audible signal. A send sidechain connection will NOT execute its input [Dsp] if it has not been executed before.
        ///
        /// A send sidechain connection will only read what exists at the input's buffer at the time of executing the output [Dsp] unit (which can be considered the 'sidechain return').
        ///
        /// For the effect developer, to accept sidechain data, the sidechain data will appear in the [DspState] struct which is passed into the read callback of a [Dsp] unit.
        ///
        /// [DspState::sidechain_data] and [DspState::sidechain_channels] will hold the mixed result of any sidechain data flowing into it.
        SendSidechain = FMOD_DSPCONNECTION_TYPE_SEND_SIDECHAIN,
    }
}
