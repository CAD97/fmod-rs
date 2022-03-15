use {
    crate::{raw::*, Error, Result, Sound, TimeUnit},
    std::ptr,
};

opaque! {
    class Channel;
}

impl Channel {
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut FMOD_CHANNEL) -> &'static Channel {
            &*(raw as *mut _)
        }

        pub fn as_raw(&self) -> *mut FMOD_CHANNEL {
            self as *const _ as *const _ as *mut _
        }
    }

    // snip
}

/// General control functionality.
impl Channel {
    // snip

    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        let result = unsafe { FMOD_Channel_GetPaused(self.as_raw(), &mut paused) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(paused != 0)
        }
    }

    // snip

    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        let result = unsafe { FMOD_Channel_IsPlaying(self.as_raw(), &mut isplaying) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(isplaying != 0)
        }
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
        let result = unsafe { FMOD_Channel_GetPosition(self.as_raw(), &mut position, postype) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(position)
        }
    }

    // snip
}

/// Information only functions
impl Channel {
    // snip

    pub fn get_current_sound(&self) -> Result<&Sound> {
        let mut sound = ptr::null_mut();
        let result = unsafe { FMOD_Channel_GetCurrentSound(self.as_raw(), &mut sound) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(unsafe { Sound::from_raw(sound) })
        }
    }

    // snip
}
