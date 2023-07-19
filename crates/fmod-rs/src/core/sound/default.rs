use {
    fmod::{raw::*, *},
    std::ops::{Bound, Range, RangeBounds, RangeInclusive},
};

/// # Defaults when played.
impl Sound {
    /// Sets the angles and attenuation levels of a 3D cone shape,
    /// for simulated occlusion which is based on direction.
    ///
    /// When [ChannelControl::set_3d_cone_orientation] is used and a 3D 'cone'
    /// is set up, attenuation will automatically occur for a sound based on the
    /// relative angle of the direction the cone is facing, vs the angle between
    /// the sound and the listener.
    pub fn set_3d_cone_settings(&self, settings: Cone3dSettings) -> Result {
        ffi!(FMOD_Sound_Set3DConeSettings(
            self.as_raw(),
            settings.inside_angle,
            settings.outside_angle,
            settings.outside_volume,
        ))
    }

    /// Retrieves the inside and outside angles of the 3D projection cone and
    /// the outside volume.
    pub fn get_3d_cone_settings(&self) -> Result<Cone3dSettings> {
        let mut cone = Cone3dSettings::default();
        ffi!(FMOD_Sound_Get3DConeSettings(
            self.as_raw(),
            &mut cone.inside_angle,
            &mut cone.outside_angle,
            &mut cone.outside_volume,
        ))?;
        Ok(cone)
    }

    // TODO: needs figuring out lifetimes
    // set_3d_custom_rolloff
    // get_3d_custom_rolloff

    /// Sets the minimum and maximum audible distance for a 3D sound.
    ///
    /// The distances are meant to simulate the 'size' of a sound. Reducing the
    /// min distance will mean the sound appears smaller in the world, and in
    /// some modes makes the volume attenuate faster as the listener moves away
    /// from the sound. Increasing the min distance simulates a larger sound in
    /// the world, and in some modes makes the volume attenuate slower as the
    /// listener moves away from the sound.
    ///
    /// The max distance will affect attenuation differently based on roll-off
    /// mode set in the mode parameter of [System::create_sound],
    /// [System::create_stream], [Sound::set_mode] or [ChannelControl::set_mode].
    ///
    /// For these modes the volume will attenuate to 0 volume (silence), when
    /// the distance from the sound is equal to or further than the max distance:
    ///
    /// - [Mode::LinearRollOff]
    /// - [Mode::LinearSquareRollOff]
    ///
    /// For these modes the volume will stop attenuating at the point of the max
    /// distance, without affecting the _rate_ of attenuation:
    ///
    /// - [Mode::InverseRollOff]
    /// - [Mode::InverseTaperedRollOff]
    ///
    /// For this mode the max distance is ignored:
    ///
    /// - [Mode::CustomRolloff]
    pub fn set_3d_min_max_distance(&self, distance: impl RangeBounds<f32>) -> Result {
        let min_distance = match distance.start_bound() {
            Bound::Included(&min_distance) => min_distance,
            Bound::Excluded(&min_distance) => min_distance,
            Bound::Unbounded => 0.0,
        };
        let max_distance = match distance.end_bound() {
            Bound::Included(&max_distance) => max_distance,
            Bound::Excluded(&max_distance) => max_distance,
            Bound::Unbounded => f32::INFINITY,
        };
        ffi!(FMOD_Sound_Set3DMinMaxDistance(
            self.as_raw(),
            min_distance,
            max_distance,
        ))
    }

    /// Retrieves the minimum and maximum audible distance for a 3D sound.
    pub fn get_3d_min_max_distance(&self) -> Result<Range<f32>> {
        let mut min = 0.0;
        let mut max = 0.0;
        ffi!(FMOD_Sound_Get3DMinMaxDistance(
            self.as_raw(),
            &mut min,
            &mut max,
        ))?;
        Ok(min..max)
    }

    /// Sets a sound's default playback attributes.
    ///
    /// When the Sound is played it will use these values without having to
    /// specify them later on a per Channel basis.
    pub fn set_defaults(&self, frequency: f32, priority: i32) -> Result {
        ffi!(FMOD_Sound_SetDefaults(self.as_raw(), frequency, priority,))
    }

    /// Retrieves a sound's default playback attributes.
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

    /// Sets or alters the mode of a sound.
    ///
    /// When calling this function, note that it will only take effect when the
    /// sound is played again with [System::play_sound]. This is the default for
    /// when the sound next plays, not a mode that will suddenly change all
    /// currently playing instances of this sound.
    ///
    /// Flags supported:
    ///
    /// - [Mode::LoopOff]
    /// - [Mode::LoopNormal]
    /// - [Mode::LoopBidi]
    /// - [Mode::HeadRelative3d]
    /// - [Mode::WorldRelative3d]
    /// - [Mode::D2]
    /// - [Mode::D3]
    /// - [Mode::InverseRollOff3D]
    /// - [Mode::LinearRollOff3D]
    /// - [Mode::LinearSquareRollOff3D]
    /// - [Mode::InverseTaperedRollOff3D]
    /// - [Mode::CustomRolloff3D]
    /// - [Mode::IgnoreGeometry3D]
    ///
    /// If [Mode::IgnoreGeometry3D] is not specified, the flag will be cleared
    /// if it was specified previously.
    ///
    /// Changing mode on an already buffered stream may not produced desired
    /// output. See [Streaming Issues](https://fmod.com/docs/2.02/api/glossary.html#streaming-issues).
    pub fn set_mode(&self, mode: Mode) -> Result {
        ffi!(FMOD_Sound_SetMode(self.as_raw(), mode.into_raw()))?;
        Ok(())
    }

    /// Retrieves the mode of a sound.
    ///
    /// The mode will be dependent on the mode set by a call to
    /// [System::create_sound], [System::create_stream] or [Sound::set_mode].
    pub fn get_mode(&self) -> Result<Mode> {
        let mut mode = Mode::default();
        ffi!(FMOD_Sound_GetMode(self.as_raw(), mode.as_raw_mut()))?;
        Ok(mode)
    }

    /// Sets the sound to loop a specified number of times before stopping if
    /// the playback mode is set to looping.
    ///
    /// If the loop count is set to -1, the sound will loop indefinitely.
    /// 0 means no loop.
    ///
    /// Changing loop count on an already buffered stream may not produced
    /// desired output. See [Streaming Issues](https://fmod.com/docs/2.02/api/glossary.html#streaming-issues).
    pub fn set_loop_count(&self, loop_count: i32) -> Result {
        ffi!(FMOD_Sound_SetLoopCount(self.as_raw(), loop_count))
    }

    /// Retrieves the sound's loop count.
    ///
    /// Unlike the [Channel] loop count function, this function simply returns
    /// the value set with [Sound::set_loop_count]. It does not decrement as it
    /// plays (especially seeing as one sound can be played multiple times).
    pub fn get_loop_count(&self) -> Result<i32> {
        let mut loop_count = 0;
        ffi!(FMOD_Sound_GetLoopCount(self.as_raw(), &mut loop_count))?;
        Ok(loop_count)
    }

    /// Sets the loop points within a sound.
    ///
    /// Changing loop points on an already buffered stream may not produced
    /// desired output. See [Streaming Issues](https://fmod.com/docs/2.02/api/glossary.html#streaming-issues).
    ///
    /// The [Sound]'s mode must be set to [Mode::LoopNormal] or [Mode::LoopBidi]
    /// for loop points to affect playback.
    pub fn set_loop_points(&self, loop_points: impl RangeBounds<Time>) -> Result {
        let loop_start = match loop_points.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => Time {
                value: start.value.saturating_add(1),
                ..start
            },
            Bound::Unbounded => Time::pcm(0),
        };
        let loop_end = match loop_points.end_bound() {
            Bound::Included(&end) => end,
            Bound::Excluded(&end) => Time {
                value: end.value.saturating_sub(1),
                ..end
            },
            Bound::Unbounded => Time::pcm(self.get_length(TimeUnit::Pcm)?.saturating_sub(1)),
        };
        ffi!(FMOD_Sound_SetLoopPoints(
            self.as_raw(),
            loop_start.value,
            loop_start.unit.into_raw(),
            loop_end.value,
            loop_end.unit.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the loop points for a sound.
    pub fn get_loop_points(&self, unit: TimeUnit) -> Result<RangeInclusive<u32>> {
        let mut start = 0;
        let mut end = 0;
        ffi!(FMOD_Sound_GetLoopPoints(
            self.as_raw(),
            &mut start,
            unit.into_raw(),
            &mut end,
            unit.into_raw(),
        ))?;
        Ok(start..=end)
    }
}
