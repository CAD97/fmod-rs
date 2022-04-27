use fmod::{raw::*, *};

opaque!(class Sound = FMOD_SOUND, FMOD_Sound_*);

/// Standard sound manipulation functions.
impl Sound {
    // snip

    pub fn get_defaults(&self) -> Result<(f32, i32)> {
        let mut frequency = 0.0;
        let mut priority = 0;
        fmod_try!(FMOD_Sound_GetDefaults(
            self.as_raw(),
            &mut frequency,
            &mut priority,
        ));
        Ok((frequency, priority))
    }

    // snip

    pub fn get_length(&self, length_type: TimeUnit) -> Result<u32> {
        let mut length = 0;
        let lengthtype = TimeUnit::into_raw(length_type);
        fmod_try!(FMOD_Sound_GetLength(self.as_raw(), &mut length, lengthtype));
        Ok(length)
    }

    // snip
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
        fmod_try!(FMOD_Sound_SetMode(self.as_raw(), mode));
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
