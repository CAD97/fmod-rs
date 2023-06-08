use {
    fmod::{raw::*, *},
    std::{ffi::c_void, ptr},
};

opaque! {
    /// The shared APIs between [Channel] and [ChannelGroup].
    weak class ChannelControl = FMOD_CHANNELCONTROL, FMOD_ChannelControl_*;
}

/// Playback
impl ChannelControl {
    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        ffi!(FMOD_Channel_IsPlaying(self.as_raw() as _, &mut isplaying))?;
        Ok(isplaying != 0)
    }

    pub fn stop(&self) -> Result {
        ffi!(FMOD_Channel_Stop(self.as_raw() as _))?;
        Ok(())
    }
}

impl ChannelControl {
    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_Channel_GetSystemObject(
            self.as_raw() as _,
            &mut system
        ))?;
        Ok(unsafe { System::from_raw(system) })
    }
}

/// General control functionality.
impl ChannelControl {
    pub fn set_paused(&self, paused: bool) -> Result {
        let paused = paused as i32;
        ffi!(FMOD_Channel_SetPaused(self.as_raw() as _, paused))?;
        Ok(())
    }

    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        ffi!(FMOD_Channel_GetPaused(self.as_raw() as _, &mut paused))?;
        Ok(paused != 0)
    }

    pub fn set_volume(&self, volume: f32) -> Result {
        ffi!(FMOD_Channel_SetVolume(self.as_raw() as _, volume))?;
        Ok(())
    }

    pub fn get_volume(&self) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_Channel_GetVolume(self.as_raw() as _, &mut volume))?;
        Ok(volume)
    }

    pub fn set_volume_ramp(&self, ramp: bool) -> Result {
        let ramp = ramp as i32;
        ffi!(FMOD_Channel_SetVolumeRamp(self.as_raw() as _, ramp))?;
        Ok(())
    }

    pub fn get_volume_ramp(&self) -> Result<bool> {
        let mut ramp = 0;
        ffi!(FMOD_Channel_GetVolumeRamp(self.as_raw() as _, &mut ramp))?;
        Ok(ramp != 0)
    }

    pub fn get_audibility(&self) -> Result<f32> {
        let mut audibility = 0.0;
        ffi!(FMOD_Channel_GetAudibility(
            self.as_raw() as _,
            &mut audibility
        ))?;
        Ok(audibility)
    }

    pub fn set_pitch(&self, pitch: f32) -> Result {
        ffi!(FMOD_Channel_SetPitch(self.as_raw() as _, pitch))?;
        Ok(())
    }

    pub fn get_pitch(&self) -> Result<f32> {
        let mut pitch = 0.0;
        ffi!(FMOD_Channel_GetPitch(self.as_raw() as _, &mut pitch))?;
        Ok(pitch)
    }

    pub fn set_mute(&self, mute: bool) -> Result {
        ffi!(FMOD_Channel_SetMute(
            self.as_raw() as _,
            if mute { 1 } else { 0 },
        ))?;
        Ok(())
    }

    pub fn get_mute(&self) -> Result<bool> {
        let mut mute = 0;
        ffi!(FMOD_Channel_GetMute(self.as_raw() as _, &mut mute))?;
        Ok(mute != 0)
    }

    pub fn set_reverb_properties(&self, instance: i32, wet: f32) -> Result {
        ffi!(FMOD_Channel_SetReverbProperties(
            self.as_raw() as _,
            instance,
            wet,
        ))?;
        Ok(())
    }

    pub fn get_reverb_properties(&self, instance: i32) -> Result<f32> {
        let mut wet = 0.0;
        ffi!(FMOD_Channel_GetReverbProperties(
            self.as_raw() as _,
            instance,
            &mut wet,
        ))?;
        Ok(wet)
    }

    pub fn set_low_pass_gain(&self, gain: f32) -> Result {
        ffi!(FMOD_Channel_SetLowPassGain(self.as_raw() as _, gain))?;
        Ok(())
    }

    pub fn get_low_pass_gain(&self) -> Result<f32> {
        let mut gain = 0.0;
        ffi!(FMOD_Channel_GetLowPassGain(self.as_raw() as _, &mut gain))?;
        Ok(gain)
    }

    pub fn set_mode(&self, mode: Mode) -> Result {
        ffi!(FMOD_Channel_SetMode(self.as_raw() as _, mode.into_raw()))?;
        Ok(())
    }

    pub fn get_mode(&self) -> Result<Mode> {
        let mut mode = 0;
        ffi!(FMOD_Channel_GetMode(self.as_raw() as _, &mut mode))?;
        Ok(Mode::from_raw(mode))
    }

    pub fn set_callback<C: ChannelControlCallback>(&self) -> Result {
        ffi!(FMOD_Channel_SetCallback(
            self.as_raw() as _,
            Some(channel_control_callback::<C>),
        ))?;
        Ok(())
    }
}

pub trait ChannelControlCallback: ChannelCallback + ChannelGroupCallback {}
impl<C: ChannelCallback + ChannelGroupCallback> ChannelControlCallback for C {}

pub(crate) unsafe extern "system" fn channel_control_callback<C: ChannelControlCallback>(
    channelcontrol: *mut FMOD_CHANNELCONTROL,
    controltype: FMOD_CHANNELCONTROL_TYPE,
    callbacktype: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    commanddata1: *mut c_void,
    commanddata2: *mut c_void,
) -> FMOD_RESULT {
    let callback = match controltype {
        FMOD_CHANNELCONTROL_CHANNEL => channel_callback::<C>,
        FMOD_CHANNELCONTROL_CHANNELGROUP => channel_group_callback::<C>,
        _ => {
            whoops!(no_panic: "unknown channel control type: {:?}", controltype);
            return FMOD_ERR_INVALID_PARAM;
        },
    };
    callback(
        channelcontrol,
        controltype,
        callbacktype,
        commanddata1,
        commanddata2,
    )
}
