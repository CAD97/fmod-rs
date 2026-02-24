use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Information.
impl Channel {
    /// Retrieves whether the Channel is being emulated by the virtual voice system.
    ///
    /// See the [Virtual Voices] guide for more information.
    ///
    /// [Virtual Voices]: https://fmod.com/docs/2.02/api/white-papers-virtual-voices.html
    pub fn is_virtual(&self) -> Result<bool> {
        let mut is_virtual = 0;
        ffi!(FMOD_Channel_IsVirtual(self.as_raw(), &mut is_virtual))?;
        Ok(is_virtual != 0)
    }

    /// Retrieves the currently playing Sound.
    pub fn get_current_sound(&self) -> Result<Option<&Sound>> {
        let mut sound = ptr::null_mut();
        ffi!(FMOD_Channel_GetCurrentSound(self.as_raw(), &mut sound))?;
        Ok(unsafe { Sound::try_from_raw(sound) })
    }

    /// Retrieves the index of this object in the System Channel pool.
    pub fn get_index(&self) -> Result<i32> {
        let mut index = 0;
        ffi!(FMOD_Channel_GetIndex(self.as_raw(), &mut index))?;
        Ok(index)
    }
}
