use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Sound Functions.
impl SoundGroup {
    /// Retrieves the current number of sounds in this sound group.
    pub fn get_num_sounds(&self) -> Result<i32> {
        let mut num_sounds = 0;
        ffi!(FMOD_SoundGroup_GetNumSounds(self.as_raw(), &mut num_sounds))?;
        Ok(num_sounds)
    }

    /// Retrieves a sound.
    ///
    /// Use [`SoundGroup::get_num_sounds`] in conjunction with this function to
    /// enumerate all sounds in a [`SoundGroup`].
    pub fn get_sound(&self, index: i32) -> Result<&Sound> {
        let mut sound = ptr::null_mut();
        ffi!(FMOD_SoundGroup_GetSound(self.as_raw(), index, &mut sound))?;
        Ok(unsafe { Sound::from_raw(sound) })
    }

    /// Retrieves the number of currently playing Channels for the SoundGroup.
    ///
    /// This routine returns the number of [`Channel`]s playing. If the
    /// [`SoundGroup`] only has one [`Sound`], and that [`Sound`] is playing
    /// twice, the figure returned will be two.
    pub fn get_num_playing(&self) -> Result<i32> {
        let mut num_playing = 0;
        ffi!(FMOD_SoundGroup_GetNumPlaying(
            self.as_raw(),
            &mut num_playing,
        ))?;
        Ok(num_playing)
    }

    /// Stops all sounds within this soundgroup.
    pub fn stop(&self) -> Result {
        ffi!(FMOD_SoundGroup_Stop(self.as_raw()))?;
        Ok(())
    }
}
