use fmod::{raw::*, *};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

/// # Filtering.
impl ChannelControl {
    /// Sets the wet / send level for a particular reverb instance.
    ///
    /// [`Channel`]s are automatically connected to all existing reverb
    /// instances due to the default wet level of 1. [`ChannelGroup`]s however
    /// will not send to any reverb by default requiring an explicit call to
    /// this function.
    ///
    /// [`ChannelGroup`] reverb is optimal for the case where you want to send 1
    /// mixed signal to the reverb, rather than a lot of individual [`Channel`]
    /// reverb sends. It is advisable to do this to reduce CPU if you have many
    /// [`Channel`]s inside a [`ChannelGroup`].
    ///
    /// When setting a wet level for a [`ChannelGroup`], any [`Channel`]s under
    /// that [`ChannelGroup`] will still have their existing sends to the
    /// reverb. To avoid this doubling up you should explicitly set the
    /// [`Channel`] wet levels to 0.
    pub fn set_reverb_properties(&self, instance: i32, wet: f32) -> Result {
        ffi!(FMOD_Channel_SetReverbProperties(
            self.as_raw() as _,
            instance,
            wet,
        ))?;
        Ok(())
    }

    /// Retrieves the wet / send level for a particular reverb instance.
    pub fn get_reverb_properties(&self, instance: i32) -> Result<f32> {
        let mut wet = 0.0;
        ffi!(FMOD_Channel_GetReverbProperties(
            self.as_raw() as _,
            instance,
            &mut wet,
        ))?;
        Ok(wet)
    }

    /// Sets the gain of the dry signal when built in lowpass / distance
    /// filtering is applied.
    ///
    /// Requires the built in lowpass to be created with
    /// [`InitFlags::ChannelLowpass`] or [`InitFlags::ChannelDistanceFilter`].
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn set_low_pass_gain(&self, gain: f32) -> Result {
        ffi!(FMOD_Channel_SetLowPassGain(self.as_raw() as _, gain))?;
        Ok(())
    }

    /// Retrieves the gain of the dry signal when built in lowpass / distance
    /// filtering is applied.
    ///
    /// Requires the built in lowpass to be created with
    /// [`InitFlags::ChannelLowpass`] or [`InitFlags::ChannelDistanceFilter`].
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn get_low_pass_gain(&self) -> Result<f32> {
        let mut gain = 0.0;
        ffi!(FMOD_Channel_GetLowPassGain(self.as_raw() as _, &mut gain))?;
        Ok(gain)
    }
}
