use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Mix properties.
impl DspConnection {
    /// Sets the connection's volume scale.
    pub fn set_mix(&self, volume: f32) -> Result<()> {
        ffi!(FMOD_DSPConnection_SetMix(self.as_raw(), volume))?;
        Ok(())
    }

    /// Retrieves the connection's volume scale.
    pub fn get_mix(&self) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_DSPConnection_GetMix(self.as_raw(), &mut volume))?;
        Ok(volume)
    }

    /// Sets a 2 dimensional pan matrix that maps the signal from
    /// input channels (columns) to output speakers (rows).
    ///
    /// This will overwrite values set via [`ChannelControl::set_pan`],
    /// [`ChannelControl::set_mix_levels_input`] and
    /// [`ChannelControl::set_mix_levels_output`].
    ///
    /// Matrix element values can be below 0 to invert a signal and above 1 to
    /// amplify the signal. Note that increasing the signal level too far may
    /// cause audible distortion.
    pub fn set_mix_matrix<M: ?Sized + AsMixMatrix>(&self, mix: &M) -> Result {
        let mix = mix.as_mix_matrix();
        ffi!(FMOD_DSPConnection_SetMixMatrix(
            self.as_raw(),
            mix.matrix().as_ptr() as _,
            mix.out_channels() as _,
            mix.in_channels() as _,
            mix.in_channel_hop() as _,
        ))?;
        Ok(())
    }

    /// Sets the default 2 dimensional pan matrix that maps the signal from
    /// input channels (columns) to output speakers (rows).
    ///
    /// This will overwrite values set via [`ChannelControl::set_pan`],
    /// [`ChannelControl::set_mix_levels_input`] and
    /// [`ChannelControl::set_mix_levels_output`].
    ///
    /// A default upmix, downmix, or unit matrix will be used.
    /// A unit matrix allows a signal to pass through unchanged.
    pub fn set_default_mix_matrix(&self) -> Result {
        ffi!(FMOD_DSPConnection_SetMixMatrix(
            self.as_raw(),
            ptr::null_mut(),
            0,
            0,
            0,
        ))?;
        Ok(())
    }

    /// Retrieves the `(in_channels, out_channels)` size of the pan matrix.
    pub fn get_mix_channels(&self) -> Result<(usize, usize)> {
        let mut out_channels = 0;
        let mut in_channels = 0;
        ffi!(FMOD_DSPConnection_GetMixMatrix(
            self.as_raw(),
            ptr::null_mut(),
            &mut out_channels,
            &mut in_channels,
            0,
        ))?;
        Ok((ix!(in_channels), ix!(out_channels)))
    }

    /// Retrieves a 2 dimensional pan matrix that maps the signal from input
    /// channels (columns) to output speakers (rows).
    ///
    /// Matrix element values can be below 0 to invert a signal and above 1 to
    /// amplify the signal. Note that increasing the signal level too far may
    /// cause audible distortion.
    pub fn get_mix_matrix<'m, M: ?Sized + AsMixMatrix>(
        &self,
        mix: &'m mut M,
    ) -> Result<&'m mut MixMatrix> {
        let mix = mix.as_mix_matrix_mut();
        let mut out_channels = 0;
        let mut in_channels = 0;
        ffi!(FMOD_DSPConnection_GetMixMatrix(
            self.as_raw(),
            ptr::null_mut(),
            &mut out_channels,
            &mut in_channels,
            0,
        ))?;
        let mix = mix.slice_mut(ix!(in_channels), ix!(out_channels));
        // ... isn't this vulnerable to TOCTOU 🙃
        ffi!(FMOD_DSPConnection_GetMixMatrix(
            self.as_raw(),
            mix.matrix_mut().as_mut_ptr() as _,
            &mut out_channels,
            &mut in_channels,
            mix.in_channel_hop() as _,
        ))?;
        Ok(mix)
    }
}
