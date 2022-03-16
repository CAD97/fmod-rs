use crate::{raw::*, Error, FmodResource, Mode, Result, TimeUnit};

opaque! {
    class Sound;
}

impl Sound {
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut FMOD_SOUND) -> &'static Sound {
            &*(raw as *mut _)
        }

        pub fn as_raw(&self) -> *mut FMOD_SOUND {
            self as *const _ as *const _ as *mut _
        }
    }

    // snip
}

unsafe impl FmodResource for Sound {
    type Raw = FMOD_SOUND;

    unsafe fn release(this: *mut Self) -> Result<()> {
        let result = FMOD_Sound_Release(this as *mut _);
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }
}

/// Standard sound manipulation functions.
impl Sound {
    // snip

    pub fn get_length(&self, length_type: TimeUnit) -> Result<u32> {
        let mut length = 0;
        let lengthtype = TimeUnit::into_raw(length_type);
        let result = unsafe { FMOD_Sound_GetLength(self.as_raw(), &mut length, lengthtype) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(length)
        }
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
        let result = unsafe { FMOD_Sound_SetMode(self.as_raw(), mode) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
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
