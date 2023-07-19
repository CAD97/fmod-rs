use fmod::{raw::*, *};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

/// # Volume levels.
impl ChannelControl {
    /// Retrieves an estimation of the output volume.
    ///
    /// Estimated volume is calculated based on 3D spatialization, occlusion,
    /// API volume levels and DSPs used.
    ///
    /// While this does not represent the actual waveform, [`Channel`]s playing
    /// FSB files will take into consideration the overall peak level of the
    /// file (if available).
    ///
    /// This value is used to determine which [`Channel`]s should be audible
    /// and which [`Channel`]s to virtualize when resources are limited.
    ///
    /// See the [Virtual Voice System][audibility-calculation] white paper for
    /// more details about how audibility is calculated.
    ///
    /// [audibility-calculation]: https://fmod.com/docs/2.02/api/white-papers-virtual-voices.html#audibility-calculation
    pub fn get_audibility(&self) -> Result<f32> {
        let mut audibility = 0.0;
        ffi!(FMOD_Channel_GetAudibility(
            self.as_raw() as _,
            &mut audibility
        ))?;
        Ok(audibility)
    }

    /// Sets the volume level.
    ///
    /// To define the volume per `Sound` use [`Sound::set_defaults`].
    ///
    /// Setting volume at a level higher than 1 can lead to distortion/clipping.
    pub fn set_volume(&self, volume: f32) -> Result {
        ffi!(FMOD_Channel_SetVolume(self.as_raw() as _, volume))?;
        Ok(())
    }

    /// Retrieves the volume level.
    ///
    /// Volume changes when not paused will be ramped to the target value to
    /// avoid a pop sound, this function allows that setting to be overridden
    /// and volume changes to be applied immediately.
    pub fn get_volume(&self) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_Channel_GetVolume(self.as_raw() as _, &mut volume))?;
        Ok(volume)
    }

    /// Sets whether volume changes are ramped or instantaneous.
    pub fn set_volume_ramp(&self, ramp: bool) -> Result {
        let ramp = ramp as i32;
        ffi!(FMOD_Channel_SetVolumeRamp(self.as_raw() as _, ramp))?;
        Ok(())
    }

    /// Retrieves whether volume changes are ramped or instantaneous.
    pub fn get_volume_ramp(&self) -> Result<bool> {
        let mut ramp = 0;
        ffi!(FMOD_Channel_GetVolumeRamp(self.as_raw() as _, &mut ramp))?;
        Ok(ramp != 0)
    }

    /// Sets the mute state.
    ///
    /// Mute is an additional control for volume, the effect of which is
    /// equivalent to setting the volume to zero.
    ///
    /// An individual mute state is kept for each object, muting a parent
    /// [`ChannelGroup`] will effectively mute this object however when queried
    /// the individual mute state is returned.
    /// [`ChannelControl::get_audibility`] can be used to calculate overall
    /// audibility for a [`Channel`] or [`ChannelGroup`].
    pub fn set_mute(&self, mute: bool) -> Result {
        ffi!(FMOD_Channel_SetMute(
            self.as_raw() as _,
            if mute { 1 } else { 0 },
        ))?;
        Ok(())
    }

    /// Retrieves the mute state.
    ///
    /// An individual mute state is kept for each object, a parent
    /// [`ChannelGroup`] being muted will effectively mute this object however
    /// when queried the individual mute state is returned.
    /// [`ChannelControl::get_audibility`] can be used to calculate overall
    /// audibility for a [`Channel`] or [`ChannelGroup`].
    pub fn get_mute(&self) -> Result<bool> {
        let mut mute = 0;
        ffi!(FMOD_Channel_GetMute(self.as_raw() as _, &mut mute))?;
        Ok(mute != 0)
    }
}
