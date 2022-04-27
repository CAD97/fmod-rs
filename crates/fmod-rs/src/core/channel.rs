use {
    fmod::{raw::*, *},
    std::ptr,
};

opaque!(weak class Channel = FMOD_CHANNEL, FMOD_Channel_*);

/// General control functionality.
impl Channel {
    // snip

    pub fn set_paused(&self, paused: bool) -> Result {
        let paused = paused as i32;
        fmod_try!(FMOD_Channel_SetPaused(self.as_raw(), paused));
        Ok(())
    }

    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        fmod_try!(FMOD_Channel_GetPaused(self.as_raw(), &mut paused));
        Ok(paused != 0)
    }

    // snip

    pub fn set_pitch(&self, pitch: f32) -> Result {
        fmod_try!(FMOD_Channel_SetPitch(self.as_raw(), pitch));
        Ok(())
    }

    pub fn get_pitch(&self) -> Result<f32> {
        let mut pitch = 0.0;
        fmod_try!(FMOD_Channel_GetPitch(self.as_raw(), &mut pitch));
        Ok(pitch)
    }

    // snip

    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        fmod_try!(FMOD_Channel_IsPlaying(self.as_raw(), &mut isplaying));
        Ok(isplaying != 0)
    }
}

/// Panning and level adjustment. Note all 'set' functions alter a final matrix,
/// this is why the only get function is `getMixMatrix`, to avoid other get
/// functions returning incorrect/obsolete values.
impl Channel {
    // snip
}

/// Clock based functionality.
impl Channel {
    pub fn get_dsp_clock(&self) -> Result<(u64, u64)> {
        let mut dsp_clock = 0;
        let mut parent_clock = 0;
        fmod_try!(FMOD_Channel_GetDSPClock(
            self.as_raw(),
            &mut dsp_clock,
            &mut parent_clock,
        ));
        Ok((dsp_clock, parent_clock))
    }

    pub fn set_delay(
        &self,
        dsp_clock_start: u64,
        dsp_clock_end: u64,
        stop_channels: bool,
    ) -> Result {
        let stop_channels = stop_channels as i32;
        fmod_try!(FMOD_Channel_SetDelay(
            self.as_raw(),
            dsp_clock_start,
            dsp_clock_end,
            stop_channels,
        ));
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
        fmod_try!(FMOD_Channel_GetPosition(
            self.as_raw(),
            &mut position,
            postype,
        ));
        Ok(position)
    }

    // snip
}

/// Information only functions
impl Channel {
    // snip

    pub fn get_current_sound(&self) -> Result<&Sound> {
        let mut sound = ptr::null_mut();
        fmod_try!(FMOD_Channel_GetCurrentSound(self.as_raw(), &mut sound));
        Ok(unsafe { Sound::from_raw(sound) })
    }

    // snip
}
