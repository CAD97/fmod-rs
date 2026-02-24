use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Relationship management.
impl Sound {
    /// Moves the sound from its existing SoundGroup to the specified sound group.
    ///
    /// By default, a sound is located in the 'master sound group'. This can be
    /// retrieved with [`System::get_master_sound_group`].
    pub fn set_sound_group(&self, sound_group: &SoundGroup) -> Result {
        ffi!(FMOD_Sound_SetSoundGroup(
            self.as_raw(),
            sound_group.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the sound's current sound group.
    pub fn get_sound_group(&self) -> Result<&SoundGroup> {
        let mut sound_group = ptr::null_mut();
        ffi!(FMOD_Sound_GetSoundGroup(self.as_raw(), &mut sound_group))?;
        Ok(unsafe { SoundGroup::from_raw(sound_group) })
    }

    /// Retrieves the number of subsounds stored within a sound.
    ///
    /// A format that has subsounds is a container format,
    /// such as FSB, DLS, MOD, S3M, XM, IT.
    pub fn get_num_sub_sounds(&self) -> Result<i32> {
        let mut num_sub_sounds = 0;
        ffi!(FMOD_Sound_GetNumSubSounds(
            self.as_raw(),
            &mut num_sub_sounds
        ))?;
        Ok(num_sub_sounds)
    }

    /// Retrieves a handle to a Sound object that is contained within the parent sound.
    ///
    /// If the sound is a stream and [`Mode::NonBlocking`] was not used, then
    /// this call will perform a blocking seek/flush to the specified subsound.
    ///
    /// If [`Mode::NonBlocking`] was used to open this sound and the sound is a
    /// stream, FMOD will do a non blocking seek/flush and set the state of the
    /// subsound to [`OpenState::Seeking`].
    ///
    /// The sound won't be ready to be used when [`Mode::NonBlocking`] is used,
    /// until the state of the sound becomes [`OpenState::Ready`] or
    /// [`OpenState::Error`].
    pub fn get_sub_sound(&self, index: i32) -> Result<&Sound> {
        let mut sub_sound = ptr::null_mut();
        ffi!(FMOD_Sound_GetSubSound(self.as_raw(), index, &mut sub_sound))?;
        Ok(unsafe { Sound::from_raw(sub_sound) })
    }

    /// Retrieves the parent Sound object that contains this subsound.
    pub fn get_sub_sound_parent(&self) -> Result<Option<&Sound>> {
        let mut parent_sound = ptr::null_mut();
        ffi!(FMOD_Sound_GetSubSoundParent(
            self.as_raw(),
            &mut parent_sound,
        ))?;
        Ok(unsafe { Sound::try_from_raw(parent_sound) })
    }
}
