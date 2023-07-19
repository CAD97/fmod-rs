use fmod::{raw::*, *};

// TODO: connections are invalidated when units are disconnected.
//       it is extremely unclear how to model this in Rust.

/// # General.
impl DspConnection {}

enum_struct! {
    /// List of connection types between 2 DSP nodes.
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
