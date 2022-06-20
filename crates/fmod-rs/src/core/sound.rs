use fmod::{raw::*, *};

opaque!(class Sound = FMOD_SOUND, FMOD_Sound_*);

/// Standard sound manipulation functions.
impl Sound {
    // snip

    pub fn get_defaults(&self) -> Result<(f32, i32)> {
        let mut frequency = 0.0;
        let mut priority = 0;
        ffi!(FMOD_Sound_GetDefaults(
            self.as_raw(),
            &mut frequency,
            &mut priority,
        ))?;
        Ok((frequency, priority))
    }

    // snip

    pub fn get_length(&self, length_type: TimeUnit) -> Result<u32> {
        let mut length = 0;
        let lengthtype = TimeUnit::into_raw(length_type);
        ffi!(FMOD_Sound_GetLength(self.as_raw(), &mut length, lengthtype))?;
        Ok(length)
    }

    // snip

    pub fn get_open_state(&self) -> Result<(OpenState, u32, bool, bool)> {
        let mut state = OpenState::zeroed();
        let mut percent_buffered = 0;
        let mut starving = 0;
        let mut disk_busy = 0;
        ffi!(FMOD_Sound_GetOpenState(
            self.as_raw(),
            state.as_raw_mut(),
            &mut percent_buffered,
            &mut starving,
            &mut disk_busy
        ))?;
        Ok((state, percent_buffered, starving != 0, disk_busy != 0))
    }
}

/// Synchronization point API. These points can come from markers embedded in
/// wav files, and can also generate channel callbacks.
impl Sound {
    // snip
}

/// Functions also in `Channel` class but here they are the 'default' to save
/// having to change it in Channel all the time.
impl Sound {
    pub fn set_mode(&self, mode: Mode) -> Result {
        let mode = Mode::into_raw(mode);
        ffi!(FMOD_Sound_SetMode(self.as_raw(), mode))?;
        Ok(())
    }

    // snip
}

/// For MOD/S3M/XM/IT/MID sequenced formats only.
impl Sound {
    // snip
}

/// Userdata set/get
impl Sound {
    // snip
}
