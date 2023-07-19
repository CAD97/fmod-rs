use {
    fmod::{raw::*, *},
    smart_default::SmartDefault,
};

/// # Processing.
impl Dsp {
    /// Sets the processing active state.
    ///
    /// If `active` is false, processing of this unit and its inputs are
    /// stopped.
    ///
    /// When created a DSP is inactive. If [`ChannelControl::add_dsp`] is used
    /// it will automatically be activated, otherwise it must be set to active
    /// manually.
    pub fn set_active(&self, active: bool) -> Result {
        ffi!(FMOD_DSP_SetActive(self.as_raw(), active as FMOD_BOOL))?;
        Ok(())
    }

    /// Retrieves the processing active state.
    ///
    /// If `active` is false, processing of this unit and its inputs are
    /// stopped.
    ///
    /// When created a DSP is inactive. If [`ChannelControl::add_dsp`] is used
    /// it will automatically be activated, otherwise it must be set to active
    /// manually.
    pub fn get_active(&self) -> Result<bool> {
        let mut active = 0;
        ffi!(FMOD_DSP_GetActive(self.as_raw(), &mut active))?;
        Ok(active != 0)
    }

    /// Sets the processing bypass state.
    ///
    /// If `bypass` is true, processing of this unit is skipped but it continues
    /// to process its inputs.
    pub fn set_bypass(&self, bypass: bool) -> Result {
        ffi!(FMOD_DSP_SetBypass(self.as_raw(), bypass as FMOD_BOOL))?;
        Ok(())
    }

    /// Retrieves the processing bypass state.
    ///
    /// If `bypass` is true, processing of this unit is skipped but it continues
    /// to process its inputs.
    pub fn get_bypass(&self) -> Result<bool> {
        let mut bypass = 0;
        ffi!(FMOD_DSP_GetBypass(self.as_raw(), &mut bypass))?;
        Ok(bypass != 0)
    }

    /// Sets the scale of the wet and dry signal components.
    pub fn set_wet_dry_mix(&self, wet_dry_mix: WetDryMix) -> Result {
        ffi!(FMOD_DSP_SetWetDryMix(
            self.as_raw(),
            wet_dry_mix.pre_wet,
            wet_dry_mix.post_wet,
            wet_dry_mix.dry,
        ))?;
        Ok(())
    }

    /// Retrieves the scale of the wet and dry signal components.
    pub fn get_wet_dry_mix(&self) -> Result<WetDryMix> {
        let mut pre_wet = 0.0;
        let mut post_wet = 0.0;
        let mut dry = 0.0;
        ffi!(FMOD_DSP_GetWetDryMix(
            self.as_raw(),
            &mut pre_wet,
            &mut post_wet,
            &mut dry,
        ))?;
        Ok(WetDryMix {
            pre_wet,
            post_wet,
            dry,
        })
    }

    /// Retrieves the idle state.
    ///
    /// A DSP is considered idle when it stops receiving input signal and all
    /// internal processing of stored input has been exhausted.
    ///
    /// Each DSP type has the potential to have differing idle behavior based on
    /// the type of effect. A reverb or echo may take a longer time to go idle
    /// after it stops receiving a valid signal, compared to an effect with a
    /// shorter tail length like an EQ filter.
    pub fn get_idle(&self) -> Result<bool> {
        let mut idle = 0;
        ffi!(FMOD_DSP_GetIdle(self.as_raw(), &mut idle))?;
        Ok(idle != 0)
    }
}

/// The scale of wet and dry DSP signal components.
///
/// The dry signal path is silent by default, because dsp effects transform the
/// input and pass the newly processed result to the output.
#[derive(Debug, Clone, Copy, PartialEq, SmartDefault)]
pub struct WetDryMix {
    /// Level of the 'Dry' (pre-processed signal) mix that is processed by the DSP.
    /// 0 = silent, 1 = full. Negative level inverts the signal.
    /// Values larger than 1 amplify the signal.
    #[default = 1.0]
    pub pre_wet: f32,
    /// Level of the 'Wet' (post-processed signal) mix that is output.
    /// 0 = silent, 1 = full. Negative level inverts the signal.
    /// Values larger than 1 amplify the signal.
    #[default = 1.0]
    pub post_wet: f32,
    /// Level of the 'Dry' (pre-processed signal) mix that is output.
    /// 0 = silent, 1 = full. Negative level inverts the signal.
    /// Values larger than 1 amplify the signal.
    #[default = 0.0]
    pub dry: f32,
}
