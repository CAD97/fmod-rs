use crate::utils::catch_user_unwind;

use {
    fmod::{raw::*, utils::decode_sbcd_u16, *},
    smart_default::SmartDefault,
    std::{borrow::Cow, ffi::c_void, ptr, time::Duration},
};

opaque! {
    /// The Digital Signal Processor is one node within a graph that transforms
    /// input audio signals to an output stream.
    ///
    /// Create with [`System::create_dsp`], [`System::create_dsp_by_type`] or
    /// [`System::create_dsp_by_plugin`].
    class Dsp = FMOD_DSP, FMOD_DSP_*;
}

/// # Connections.
impl Dsp {
    /// Adds a DSP unit as an input to this object.
    ///
    /// When a DSP has multiple inputs the signals are automatically mixed
    /// together, sent to the unit's output(s).
    ///
    /// The returned connection will remain valid until the units are
    /// disconnected.
    pub fn add_input(&self, input: &Dsp, kind: DspConnectionType) -> Result<&DspConnection> {
        let mut connection = ptr::null_mut();
        ffi!(FMOD_DSP_AddInput(
            self.as_raw(),
            input.as_raw(),
            &mut connection,
            kind.into_raw(),
        ))?;
        Ok(unsafe { DspConnection::from_raw(connection) })
    }

    /// Retrieves the DSP unit at the specified index in the input list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the input list is correct, avoid this during time sensitive operations.
    ///
    /// The returned connection will remain valid until the units are
    /// disconnected.
    pub fn get_input(&self, index: i32) -> Result<(&Dsp, &DspConnection)> {
        let mut dsp = ptr::null_mut();
        let mut connection = ptr::null_mut();
        ffi!(FMOD_DSP_GetInput(
            self.as_raw(),
            index,
            &mut dsp,
            &mut connection,
        ))?;
        Ok(unsafe { (Dsp::from_raw(dsp), DspConnection::from_raw(connection)) })
    }

    /// Retrieves the DSP unit at the specified index in the output list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the output list is correct, avoid this during time sensitive operations.
    ///
    /// The returned connection will remain valid until the units are
    /// disconnected.
    pub fn get_output(&self, index: i32) -> Result<(&Dsp, &DspConnection)> {
        let mut dsp = ptr::null_mut();
        let mut connection = ptr::null_mut();
        ffi!(FMOD_DSP_GetOutput(
            self.as_raw(),
            index,
            &mut dsp,
            &mut connection,
        ))?;
        Ok(unsafe { (Dsp::from_raw(dsp), DspConnection::from_raw(connection)) })
    }

    /// Retrieves the number of DSP units in the input list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the input list is correct, avoid this during time sensitive operations.
    pub fn get_num_inputs(&self) -> Result<i32> {
        let mut num_inputs = 0;
        ffi!(FMOD_DSP_GetNumInputs(self.as_raw(), &mut num_inputs))?;
        Ok(num_inputs)
    }

    /// Retrieves the number of DSP units in the output list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the output list is correct, avoid this during time sensitive operations.
    pub fn get_num_outputs(&self) -> Result<i32> {
        let mut num_outputs = 0;
        ffi!(FMOD_DSP_GetNumOutputs(self.as_raw(), &mut num_outputs))?;
        Ok(num_outputs)
    }

    /// Disconnects all inputs and outputs.
    ///
    /// This is a convenience function that is faster than disconnecting all
    /// inputs and outputs individually.
    pub fn disconnect_all(&self) -> Result {
        ffi!(FMOD_DSP_DisconnectAll(
            self.as_raw(),
            /* inputs */ true as FMOD_BOOL,
            /* outputs */ true as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Disconnects all inputs.
    ///
    /// This is a convenience function that is faster than disconnecting all
    /// inputs individually.
    pub unsafe fn disconnect_all_inputs(&self) -> Result {
        ffi!(FMOD_DSP_DisconnectAll(
            self.as_raw(),
            /* inputs */ true as FMOD_BOOL,
            /* outputs */ false as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Disconnects all outputs.
    ///
    /// This is a convenience function that is faster than disconnecting all
    /// outputs individually.
    pub unsafe fn disconnect_all_outputs(&self) -> Result {
        ffi!(FMOD_DSP_DisconnectAll(
            self.as_raw(),
            /* inputs */ false as FMOD_BOOL,
            /* outputs */ true as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Disconnect the specified input DSP.
    ///
    /// If `target` had only one output, after this operation that entire sub
    /// graph will no longer be connected to the DSP network.
    pub fn disconnect_from_input(&self, target: &Dsp) -> Result {
        ffi!(FMOD_DSP_DisconnectFrom(
            self.as_raw(),
            target.as_raw(),
            ptr::null_mut(),
        ))?;
        Ok(())
    }

    /// Disconnect the specified output DSP.
    ///
    /// If `self` had only one output, after this operation this entire sub
    /// graph will no longer be connected to the DSP network.
    pub fn disconnect_from_output(&self, target: &Dsp) -> Result {
        ffi!(FMOD_DSP_DisconnectFrom(
            target.as_raw(),
            self.as_raw(),
            ptr::null_mut(),
        ))?;
        Ok(())
    }
}

/// # Parameters.
impl Dsp {
    // TODO: Plugin interface.
}

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

/// # Metering.
impl Dsp {
    // TODO: Plugin interface.
}

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

/// # General.
impl Dsp {
    // TODO: show_config_dialog

    /// Reset a DSPs internal state ready for new input signal.
    ///
    /// This will clear all internal state derived from input signal while
    /// retaining any set parameter values. The intended use of the function is
    /// to avoid audible artifacts if moving the DSP from one part of the DSP
    /// network to another.
    pub fn reset(&self) -> Result {
        ffi!(FMOD_DSP_Reset(self.as_raw()))?;
        Ok(())
    }

    /// Retrieves the pre-defined type of a FMOD registered DSP unit.
    ///
    /// This is only valid for built in FMOD effects. Any user plugins will
    /// simply return [`DspType::Unknown`].
    pub fn get_type(&self) -> Result<DspType> {
        let mut dsp_type = 0;
        ffi!(FMOD_DSP_GetType(self.as_raw(), &mut dsp_type))?;
        Ok(DspType::from_raw(dsp_type))
    }

    /// Retrieves information about this DSP unit.
    pub fn get_info(&self) -> Result<DspInfo> {
        let mut info = DspInfo::default();
        ffi!(FMOD_DSP_GetInfo(
            self.as_raw(),
            &mut info.name as *mut _ as *mut _,
            &mut info.version,
            &mut info.channels,
            &mut info.config_width,
            &mut info.config_height,
        ))?;
        Ok(info)
    }

    /// Retrieves statistics on the mixer thread CPU usage for this unit.
    ///
    /// [`InitFlags::ProfileEnable`] with [`System::init`] is required to call
    /// this function.
    pub fn get_cpu_usage(&self) -> Result<CpuDurations> {
        let mut exclusive = 0;
        let mut inclusive = 0;
        ffi!(FMOD_DSP_GetCPUUsage(
            self.as_raw(),
            &mut exclusive,
            &mut inclusive,
        ))?;
        Ok(CpuDurations {
            exclusive: Duration::from_micros(exclusive as u64),
            inclusive: Duration::from_micros(inclusive as u64),
        })
    }

    // TODO: set_user_data, get_user_data

    pub fn set_callback<C: DspCallback>(&self) -> Result {
        ffi!(FMOD_DSP_SetCallback(self.as_raw(), Some(dsp_callback::<C>)))?;
        Ok(())
    }

    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_DSP_GetSystemObject(self.as_raw(), &mut system))?;
        Ok(unsafe { System::from_raw(system) })
    }
}

/// Information about a DSP unit.
#[derive(Debug, Clone, Copy, Default)]
pub struct DspInfo {
    name: [u8; 32],
    version: u32,
    /// Number of channels this unit processes where 0 represents "any".
    pub channels: i32,
    /// Configuration dialog box width where 0 represents "no dialog box".
    pub config_width: i32,
    /// Configuration dialog box height where 0 represents "no dialog box".
    pub config_height: i32,
}

impl DspInfo {
    /// The name of this unit.
    pub fn name(&self) -> Cow<'_, str> {
        // Don't use CStr to be resilient to a lack of null termination.
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(32);
        String::from_utf8_lossy(&self.name[..len])
    }

    /// Version number of this unit, usually formatted as hex AAAABBBB where the
    /// AAAA is the major version number and the BBBB is the minor version
    /// number.
    pub fn raw_version(&self) -> u32 {
        self.version
    }

    /// (Major, minor) version number of this unit, decoded from the packed
    /// simple binary coded decimal representation.
    pub fn version(&self) -> (u16, u16) {
        let minor = self.version as u16;
        let major = (self.version >> 16) as u16;
        (decode_sbcd_u16(major), decode_sbcd_u16(minor))
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuDurations {
    pub exclusive: Duration,
    pub inclusive: Duration,
}

/// Callbacks called by DSPs.
///
/// Callbacks are called from the game thread when set from the Core API or
/// Studio API in synchronous mode, and from the Studio Update Thread when in
/// default / async mode.
pub unsafe trait DspCallback {
    /// Called when a DSP's data parameter can be released.
    ///
    /// The callback should free the data pointer if it is no longer required.
    unsafe fn data_parameter_release(dsp: &Dsp, data: *mut [u8], index: i32) -> Result {
        let _ = (dsp, data, index);
        Ok(())
    }
}

pub(crate) unsafe extern "system" fn dsp_callback<C: DspCallback>(
    dsp: *mut FMOD_DSP,
    kind: FMOD_DSP_CALLBACK_TYPE,
    data: *mut c_void,
) -> FMOD_RESULT {
    let dsp = Dsp::from_raw(dsp);
    match kind {
        FMOD_DSP_CALLBACK_DATAPARAMETERRELEASE => {
            let data = data as *mut FMOD_DSP_DATA_PARAMETER_INFO;
            let index = (*data).index;
            let data = ptr::slice_from_raw_parts_mut((*data).data.cast(), ix!((*data).length));
            catch_user_unwind(|| C::data_parameter_release(dsp, data, index)).into_raw()
        },
        _ => {
            whoops!(no_panic: "unknown dsp callback type {:?}", kind);
            FMOD_ERR_INVALID_PARAM
        },
    }
}

raw! {
    enum_struct! {
        /// Types of callbacks called by DSPs.
        ///
        /// Callbacks are called from the game thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        pub enum DspCallbackType: FMOD_DSP_CALLBACK_TYPE {
            /// Called when a DSP's data parameter can be released.
            DataParameterInfo = FMOD_DSP_CALLBACK_DATAPARAMETERRELEASE,
        }
    }
}
