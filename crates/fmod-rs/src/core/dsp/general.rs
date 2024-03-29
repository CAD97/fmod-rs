use {
    crate::utils::{catch_user_unwind, decode_sbcd_u16},
    fmod::{raw::*, *},
    std::{borrow::Cow, ffi::c_void, ptr, time::Duration},
};

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

    raw! {
        pub unsafe fn raw_release(this: *mut FMOD_DSP) -> FMOD_RESULT {
            FMOD_DSP_Release(this)
        }
    }

    /// Retrieves the pre-defined type of a FMOD registered DSP unit.
    ///
    /// This is only valid for built in FMOD effects. Any user plugins will
    /// simply return [`DspType::Unknown`].
    pub fn get_type(&self) -> Result<DspType> {
        let mut kind = DspType::zeroed();
        ffi!(FMOD_DSP_GetType(self.as_raw(), kind.as_raw_mut()))?;
        Ok(kind)
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

    /// Sets the callback for DSP notifications.
    pub fn set_callback<C: DspCallback>(&self) -> Result {
        ffi!(FMOD_DSP_SetCallback(self.as_raw(), Some(dsp_callback::<C>)))?;
        Ok(())
    }

    /// Retrieves the parent System object.
    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_DSP_GetSystemObject(self.as_raw(), &mut system))?;
        Ok(unsafe { System::from_raw(system) })
    }
}

/// Thread CPU usage statistics for a DSP unit.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuDurations {
    /// CPU time spent processing just this unit during the last mixer update.
    pub exclusive: Duration,
    /// CPU time spent processing this unit and all of its input during the last mixer update.
    pub inclusive: Duration,
}

/// Callbacks called by DSPs.
///
/// Callbacks are called from the game thread when set from the Core API or
/// Studio API in synchronous mode, and from the Studio Update Thread when in
/// default / async mode.
pub trait DspCallback {
    /// Called when a DSP's data parameter can be released.
    ///
    /// The callback should free the data pointer if it is no longer required.
    ///
    /// # Safety
    ///
    /// The data pointer has the same provenance as when it was initially set.
    //
    // TODO: does this mean that setting DSP data parameters is by-ref?
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
    catch_user_unwind(|| {
        let dsp = Dsp::from_raw(dsp);
        let kind = DspCallbackType::try_from_raw(kind)?;
        match kind {
            DspCallbackType::DataParameterRelease => {
                let data = data as *mut FMOD_DSP_DATA_PARAMETER_INFO;
                let index = (*data).index;
                let data = ptr::slice_from_raw_parts_mut((*data).data.cast(), ix!((*data).length));
                C::data_parameter_release(dsp, data, index)
            },
        }
    })
    .into_raw()
}

raw! {
    fmod_enum! {
        /// Types of callbacks called by DSPs.
        ///
        /// Callbacks are called from the game thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        pub enum DspCallbackType: FMOD_DSP_CALLBACK_TYPE
        where const { self < FMOD_DSP_CALLBACK_MAX }
        {
            /// Called when a DSP's data parameter can be released.
            DataParameterRelease = FMOD_DSP_CALLBACK_DATAPARAMETERRELEASE,
        }
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
