use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Channel format.
impl Dsp {
    /// Sets the PCM input format this DSP will receive when processing.
    ///
    /// Setting the number of channels on a unit will force either a down or up
    /// mix to that channel count before processing the DSP read/process
    /// callback.
    pub fn set_channel_format(
        &self,
        num_channels: i32,
        source_speaker_mode: SpeakerMode,
    ) -> Result {
        ffi!(FMOD_DSP_SetChannelFormat(
            self.as_raw(),
            /* channel_mask */ 0, // deprecated
            num_channels,
            source_speaker_mode.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the PCM input format this DSP will receive when processing.
    pub fn get_channel_format(&self) -> Result<(i32, SpeakerMode)> {
        let mut num_channels = 0;
        let mut source_speaker_mode = 0;
        ffi!(FMOD_DSP_GetChannelFormat(
            self.as_raw(),
            /* channel_mask */ ptr::null_mut(), // deprecated
            &mut num_channels,
            &mut source_speaker_mode,
        ))?;
        Ok((num_channels, SpeakerMode::from_raw(source_speaker_mode)))
    }

    pub fn get_output_channel_format(
        &self,
        in_channels: i32,
        in_speaker_mode: SpeakerMode,
    ) -> Result<(i32, SpeakerMode)> {
        let mut out_channels = 0;
        let mut out_speaker_mode = 0;
        ffi!(FMOD_DSP_GetOutputChannelFormat(
            self.as_raw(),
            /* channel_mask */ 0, // deprecated
            in_channels,
            in_speaker_mode.into_raw(),
            /* channel_mask */ ptr::null_mut(), // deprecated
            &mut out_channels,
            &mut out_speaker_mode,
        ))?;
        Ok((out_channels, SpeakerMode::from_raw(out_speaker_mode)))
    }
}
