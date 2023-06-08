use {
    fmod::{raw::*, utils::catch_user_unwind, *},
    std::{ffi::c_void, ops::Deref, panic::AssertUnwindSafe, ptr},
};

opaque!(weak class Channel = FMOD_CHANNEL, FMOD_Channel_*);

impl Deref for Channel {
    type Target = ChannelControl;
    fn deref(&self) -> &Self::Target {
        unsafe { ChannelControl::from_raw(self.as_raw() as _) }
    }
}

/// General control functionality.
impl Channel {
    /// Sets the gain of the dry signal when built in lowpass / distance
    /// filtering is applied.
    ///
    /// Requires the built in lowpass to be created with
    /// [InitFlags::ChannelLowpass] or [InitFlags::ChannelDistanceFilter].
    pub fn set_low_pass_gain(&self, gain: f32) -> Result {
        ffi!(FMOD_Channel_SetLowPassGain(self.as_raw(), gain))?;
        Ok(())
    }

    /// Retrieves the gain of the dry signal when built in lowpass / distance
    /// filtering is applied.
    ///
    /// Requires the built in lowpass to be created with
    /// [InitFlags::ChannelLowpass] or [InitFlags::ChannelDistanceFilter].
    pub fn get_low_pass_gain(&self) -> Result<f32> {
        let mut gain = 0.0;
        ffi!(FMOD_Channel_GetLowPassGain(self.as_raw(), &mut gain))?;
        Ok(gain)
    }

    /// Sets the callback for ChannelControl level notifications.
    pub fn set_callback<C: ChannelCallback>(&self) -> Result {
        ffi!(FMOD_Channel_SetCallback(
            self.as_raw(),
            Some(channel_callback::<C>),
        ))?;
        Ok(())
    }
}

pub trait ChannelCallback {
    /// Called when a sound ends.
    fn end(channel: &Channel) {
        let _ = channel;
    }

    /// Called when a [Channel] is made virtual or real.
    fn virtual_voice(channel: &Channel, is_virtual: bool) {
        let _ = (channel, is_virtual);
    }

    /// Called when a sync point is encountered.
    /// Can be from wav file markers or user added.
    fn sync_point(channel: &Channel, point: i32) {
        let _ = (channel, point);
    }

    /// Called when geometry occlusion values are calculated.
    /// Can be used to clamp or change the value.
    fn occlusion(channel: &Channel, direct: &mut f32, reverb: &mut f32) {
        let _ = (channel, direct, reverb);
    }
}

pub(crate) unsafe extern "system" fn channel_callback<C: ChannelCallback>(
    channelcontrol: *mut FMOD_CHANNELCONTROL,
    controltype: FMOD_CHANNELCONTROL_TYPE,
    callbacktype: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    commanddata1: *mut c_void,
    commanddata2: *mut c_void,
) -> FMOD_RESULT {
    if controltype != FMOD_CHANNELCONTROL_CHANNEL {
        whoops!(no_panic: "channel callback called with channel group");
        return FMOD_ERR_INVALID_PARAM;
    }

    let channel = AssertUnwindSafe(Channel::from_raw(channelcontrol as *mut FMOD_CHANNEL));
    match callbacktype {
        FMOD_CHANNELCONTROL_CALLBACK_END => {
            catch_user_unwind(|| C::end(&channel));
        },
        FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE => {
            let is_virtual = commanddata1 as i32 != 0;
            catch_user_unwind(|| C::virtual_voice(&channel, is_virtual));
        },
        FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT => {
            let point = commanddata1 as i32;
            catch_user_unwind(|| C::sync_point(&channel, point));
        },
        FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION => {
            let mut direct = AssertUnwindSafe(&mut *(commanddata1 as *mut f32));
            let mut reverb = AssertUnwindSafe(&mut *(commanddata2 as *mut f32));
            catch_user_unwind(move || C::occlusion(&channel, &mut direct, &mut reverb));
        },
        _ => {
            whoops!(no_panic: "unknown channel callback type {:?}", callbacktype);
            return FMOD_ERR_INVALID_PARAM;
        },
    }

    FMOD_OK
}

/// Panning and level adjustment. Note all 'set' functions alter a final matrix,
/// this is why the only get function is `get_mix_matrix`, to avoid other get
/// functions returning incorrect/obsolete values.
impl Channel {
    pub fn set_pan(&self, pan: f32) -> Result {
        ffi!(FMOD_Channel_SetPan(self.as_raw(), pan))?;
        Ok(())
    }

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
            self.as_raw(),
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

    pub fn set_mix_levels_input(&self, levels: &[f32]) -> Result {
        ffi!(FMOD_Channel_SetMixLevelsInput(
            self.as_raw(),
            levels.as_ptr().cast_mut(),
            levels.len() as i32,
        ))?;
        Ok(())
    }

    pub fn set_mix_matrix(
        &self,
        matrix: &mut [f32],
        out_channels: i32,
        in_channels: i32,
        in_channel_hop: i32,
    ) -> Result {
        ffi!(FMOD_Channel_SetMixMatrix(
            self.as_raw(),
            matrix.as_mut_ptr(),
            out_channels,
            in_channels,
            in_channel_hop,
        ))?;
        Ok(())
    }
}

/// Clock based functionality.
impl Channel {
    pub fn get_dsp_clock(&self) -> Result<(u64, u64)> {
        let mut dsp_clock = 0;
        let mut parent_clock = 0;
        ffi!(FMOD_Channel_GetDSPClock(
            self.as_raw(),
            &mut dsp_clock,
            &mut parent_clock,
        ))?;
        Ok((dsp_clock, parent_clock))
    }

    pub fn set_delay(
        &self,
        dsp_clock_start: u64,
        dsp_clock_end: u64,
        stop_channels: bool,
    ) -> Result {
        let stop_channels = stop_channels as i32;
        ffi!(FMOD_Channel_SetDelay(
            self.as_raw(),
            dsp_clock_start,
            dsp_clock_end,
            stop_channels,
        ))?;
        Ok(())
    }

    // snip
}

/// DSP effects.
impl Channel {
    // snip
}

/// 3D functionality.
impl Channel {
    // snip
}

/// Userdata set/get.
impl Channel {
    // snip
}

/// Specific control functionality.
impl Channel {
    // snip

    pub fn get_position(&self, pos_type: TimeUnit) -> Result<u32> {
        let mut position = 0;
        let postype = TimeUnit::into_raw(pos_type);
        ffi!(FMOD_Channel_GetPosition(
            self.as_raw(),
            &mut position,
            postype,
        ))?;
        Ok(position)
    }

    // snip
}

/// Information only functions
impl Channel {
    // snip

    pub fn get_current_sound(&self) -> Result<&Sound> {
        let mut sound = ptr::null_mut();
        ffi!(FMOD_Channel_GetCurrentSound(self.as_raw(), &mut sound))?;
        Ok(unsafe { Sound::from_raw(sound) })
    }

    // snip
}
