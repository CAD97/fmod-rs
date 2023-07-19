use fmod::{raw::*, *};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

/// # Panning and level adjustment.
impl ChannelControl {
    /// Sets the left/right pan level.
    ///
    /// This is a convenience function to avoid passing a matrix, it will
    /// overwrite values set via [`ChannelControl::set_mix_levels_input`],
    /// [`ChannelControl::set_mix_levels_output`] and
    /// [`ChannelControl::set_mix_matrix`].
    ///
    /// Mono inputs are panned from left to right using constant power panning
    /// (non linear fade). Stereo and greater inputs will isolate the front left
    /// and right input channels and fade them up and down based on the pan
    /// value (silencing other channels). The output channel count will always
    /// match the [`System`] speaker mode set via
    /// [`System::set_software_format`].
    ///
    /// If the System is initialized with [`SpeakerMode::Raw`] calling this
    /// function will produce silence.
    pub fn set_pan(&self, pan: f32) -> Result {
        ffi!(FMOD_Channel_SetPan(self.as_raw() as _, pan))?;
        Ok(())
    }

    /// Sets the incoming volume level for each channel of a multi-channel signal.
    ///
    /// This is a convenience function to avoid passing a matrix,
    /// it will overwrite values set via [ChannelControl::set_pan],
    /// [ChannelControl::set_mix_levels_output], and
    /// [ChannelControl::set_mix_matrix].
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn set_mix_levels_input(&self, levels: &[f32]) -> Result {
        ffi!(FMOD_Channel_SetMixLevelsInput(
            self.as_raw() as _,
            levels.as_ptr() as _,
            levels.len() as _,
        ))?;
        Ok(())
    }

    /// Sets the outgoing volume levels for each speaker.
    ///
    /// Specify the level for a given output speaker, if the channel count of
    /// the input and output do not match, channels will be up/down mixed as
    /// appropriate to approximate the given speaker values. For example stereo
    /// input with 5.1 output will use the `center` parameter to distribute
    /// signal to the center speaker from the front left and front right
    /// channels.
    ///
    /// This is a convenience function to avoid passing a matrix,
    /// it will overwrite values set via [ChannelControl::set_pan],
    /// [ChannelControl::set_mix_levels_input], and
    /// [ChannelControl::set_mix_matrix].
    ///
    /// The output channel count will always match the System speaker mode set
    /// via [System::set_software_format].
    ///
    /// If the System is initialized with [InitFlags::SpeakerModeRaw] calling
    /// this function will produce silence.
    #[allow(clippy::too_many_arguments)] // it's on FMOD, not me
    pub fn set_mix_levels_output(
        &self,
        front_left: f32,
        front_right: f32,
        center: f32,
        lfe: f32,
        surround_left: f32,
        surround_right: f32,
        back_left: f32,
        back_right: f32,
    ) -> Result {
        ffi!(FMOD_Channel_SetMixLevelsOutput(
            self.as_raw() as _,
            front_left,
            front_right,
            center,
            lfe,
            surround_left,
            surround_right,
            back_left,
            back_right,
        ))?;
        Ok(())
    }

    // TODO: needs figuring out how to handle matrices
    // set_mix_matrix
    // get_mix_matrix
}
