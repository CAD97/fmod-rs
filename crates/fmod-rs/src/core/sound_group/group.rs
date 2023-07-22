use fmod::{raw::*, *};

/// # Group Functions.
impl SoundGroup {
    /// Sets the maximum number of playbacks to be audible at once in a sound
    /// group. -1 denotes unlimited.
    ///
    /// If playing instances of sounds in this group equal or exceed number
    /// specified here, attempts to play more of the sounds with be met with
    /// [`Error::MaxAudible`] by default. Use
    /// [`SoundGroup::set_max_audible_behavior`] to change the way the sound
    /// playback behaves when too many sounds are playing. Muting, failing and
    /// stealing behaviors can be specified. See [`SoundGroupBehavior`].
    ///
    /// [`SoundGroup::get_num_playing`] can be used to determine how many
    /// instances of the sounds in the SoundGroup are currently playing.
    pub fn set_max_audible(&self, max_audible: i32) -> Result {
        ffi!(FMOD_SoundGroup_SetMaxAudible(self.as_raw(), max_audible))?;
        Ok(())
    }

    /// Retrieves the maximum number of playbacks to be audible at once in a
    /// sound group.
    pub fn get_max_audible(&self) -> Result<i32> {
        let mut max_audible = 0;
        ffi!(FMOD_SoundGroup_GetMaxAudible(
            self.as_raw(),
            &mut max_audible,
        ))?;
        Ok(max_audible)
    }

    /// This function changes the way the sound playback behaves when too many
    /// sounds are playing in a soundgroup.
    pub fn set_max_audible_behavior(&self, behavior: SoundGroupBehavior) -> Result {
        ffi!(FMOD_SoundGroup_SetMaxAudibleBehavior(
            self.as_raw(),
            behavior.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the current max audible behavior.
    pub fn get_max_audible_behavior(&self) -> Result<SoundGroupBehavior> {
        let mut behavior = SoundGroupBehavior::zeroed();
        ffi!(FMOD_SoundGroup_GetMaxAudibleBehavior(
            self.as_raw(),
            behavior.as_raw_mut(),
        ))?;
        Ok(behavior)
    }

    /// Sets a mute fade time.
    ///
    /// If a mode besides [`SoundGroupBehavior::Mute`] is used, the fade speed
    /// is ignored.
    ///
    /// When more sounds are playing in a SoundGroup than are specified with
    /// [`SoundGroup::set_max_audible`], the least important [`Sound`] (ie
    /// lowest priority / lowest audible volume due to 3D position, volume etc)
    /// will fade to silence if [`SoundGroupBehavior::Mute`] is used, and any
    /// previous sounds that were silent because of this rule will fade in if
    /// they are more important.
    pub fn set_mute_fade_speed(&self, speed: f32) -> Result {
        ffi!(FMOD_SoundGroup_SetMuteFadeSpeed(self.as_raw(), speed))?;
        Ok(())
    }

    /// Retrieves the current mute fade time.
    pub fn get_mute_fade_speed(&self) -> Result<f32> {
        let mut speed = 0.0;
        ffi!(FMOD_SoundGroup_GetMuteFadeSpeed(self.as_raw(), &mut speed))?;
        Ok(speed)
    }

    /// Sets the volume of the sound group.
    ///
    /// Scales the volume of all [`Channels`] playing [`Sound`]s in this
    /// [`SoundGroup`].
    pub fn set_volume(&self, volume: f32) -> Result {
        ffi!(FMOD_SoundGroup_SetVolume(self.as_raw(), volume))?;
        Ok(())
    }

    /// Retrieves the volume of the sound group.
    pub fn get_volume(&self) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_SoundGroup_GetVolume(self.as_raw(), &mut volume))?;
        Ok(volume)
    }
}

fmod_enum! {
    /// Values specifying behavior when a sound group's max audible value is exceeded.
    ///
    /// When using [SoundGroupBehavior::Mute], [SoundGroup::set_mute_fade_speed] can be used to stop a sudden transition.
    /// Instead, the time specified will be used to cross fade between the sounds that go silent and the ones that become audible.
    #[derive(Default)]
    pub enum SoundGroupBehavior: FMOD_SOUNDGROUP_BEHAVIOR {
        #[default]
        /// Excess sounds will fail when calling [System::play_sound].
        Fail        = FMOD_SOUNDGROUP_BEHAVIOR_FAIL,
        /// Excess sounds will begin mute and will become audible when sufficient sounds are stopped.
        Mute        = FMOD_SOUNDGROUP_BEHAVIOR_MUTE,
        /// Excess sounds will steal from the quietest [Sound] playing in the group.
        StealLowest = FMOD_SOUNDGROUP_BEHAVIOR_STEALLOWEST,
    }
}
