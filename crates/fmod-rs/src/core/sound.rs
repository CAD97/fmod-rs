use std::ptr;

use {
    fmod::{raw::*, utils::fmod_get_string, *},
    std::{
        mem,
        ops::{Bound, Range, RangeBounds, RangeInclusive},
    },
};

opaque! {
    /// Container for [sample data] that can be played on a [Channel].
    ///
    /// Create with [System::create_sound] or [System::create_stream].
    ///
    /// [sample data]: https://fmod.com/docs/2.02/api/glossary.html#sample-data
    class Sound = FMOD_SOUND, FMOD_Sound_*;
}

/// # Format information.
impl Sound {
    /// Retrieves the name of a sound.
    ///
    /// If [Mode::LowMem] has been specified in [System::create_sound], this
    /// function will return `"(null)"`.
    pub fn get_name(&self, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_Sound_GetName(
                    self.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as i32
                ))
            })
        }
    }

    /// Returns format information about the sound.
    pub fn get_format(&self) -> Result<SoundFormatInfo> {
        let mut info = SoundFormatInfo::default();
        ffi!(FMOD_Sound_GetFormat(
            self.as_raw(),
            info.kind.as_raw_mut(),
            info.format.as_raw_mut(),
            &mut info.channels,
            &mut info.bits_per_sample,
        ))?;
        Ok(info)
    }

    /// Retrieves the length using the specified time unit.
    ///
    /// `length_type` must be valid for the file format. For example, an MP3
    /// file does not support [TimeUnit::ModOrder].
    ///
    /// A length of `u32::MAX` means it is of unlimited length, such as an
    /// internet radio stream or MOD/S3M/XM/IT file which may loop forever.
    ///
    /// **Note:** Using a VBR (Variable Bit Rate) source that does not have
    /// metadata containing its accurate length (such as un-tagged MP3 or
    /// MOD/S3M/XM/IT) may return inaccurate length values.
    /// For these formats, use [Mode::AccurateTime] when creating the sound.
    /// This will cause a slight delay and memory increase, as FMOD will scan
    /// the whole during creation to find the correct length. This flag also
    /// creates a seek table to enable sample accurate seeking.
    pub fn get_length(&self, length_type: TimeUnit) -> Result<u32> {
        let mut length = 0;
        ffi!(FMOD_Sound_GetLength(
            self.as_raw(),
            &mut length,
            length_type.into_raw(),
        ))?;
        Ok(length)
    }

    /// Retrieves the number of metadata tags.
    ///
    /// 'Tags' are metadata stored within a sound file. These can be things like
    /// a song's name, composer etc.
    pub fn get_num_tags(&self) -> Result<i32> {
        let mut num_tags = 0;
        ffi!(FMOD_Sound_GetNumTags(
            self.as_raw(),
            &mut num_tags,
            ptr::null_mut(),
        ))?;
        Ok(num_tags)
    }

    /// Retrieves the number of metadata tags updated since this function was
    /// last called.
    ///
    /// This could be periodically checked to see if new tags are available in
    /// certain circumstances. This might be the case with internet based
    /// streams (i.e. shoutcast or icecast) where the name of the song or other
    /// attributes might change.
    pub fn get_num_tags_updated(&self) -> Result<i32> {
        // XXX: Does `GetNumTags(sound, &numtags, nullptr)` reset this value?
        let mut num_tags_updated = 0;
        ffi!(FMOD_Sound_GetNumTags(
            self.as_raw(),
            ptr::null_mut(),
            &mut num_tags_updated,
        ))?;
        Ok(num_tags_updated)
    }

    /// Retrieves a metadata tag.
    ///
    /// 'Tags' are metadata stored within a sound file. These can be things like
    /// a song's name, composer etc.
    ///
    /// The number of tags available can be found with [Sound::get_num_tags].
    ///
    /// The way to display or retrieve tags can be done in 3 different ways:
    ///
    /// - All tags can be continuously retrieved by looping
    ///   `0..Sound::get_num_tags()`. Updated tags will refresh automatically,
    ///   and the 'updated' member of the [Tag] structure will be set to true if
    ///   a tag has been updated, due to something like a netstream changing the
    ///   song name for example.
    /// - Tags can be retrieved by specifying -1 as the index and only updating
    ///   tags that are returned. If all tags are retrieved and this function is
    ///   called the function will return an error of [Error::TagNotFound].
    /// - Specific tags can be retrieved by specifying a name parameter. The
    ///   index can be 0 based or -1 in the same fashion as described previously.
    ///
    /// Note with netstreams an important consideration must be made between
    /// songs, a tag may occur that changes the playback rate of the song.
    /// It is up to the user to catch this and reset the playback rate with
    /// [Channel::set_frequency]. A sample rate change will be signalled with
    /// a tag of type [TagType::Fmod].
    ///
    /// ```no_run
    /// # macro_rules! yeet { ($e:expr) => (return Err($e)?) }
    /// # let system = fmod::System::new()?;
    /// # let sound = system.create_sound(fmod::cstr8!("drumloop.wav"), fmod::Mode::Default)?;
    /// # let channel = system.play_sound(&sound, None, false)?;
    /// loop {
    ///     let tag = match sound.get_tag(None, -1) {
    ///         Err(fmod::Error::TagNotFound) => break,
    ///         tag => tag?,
    ///     };
    ///     if tag.kind == fmod::TagType::Fmod {
    ///         // When a song changes, the sample rate may also change, so compensate here.
    ///         if tag.name == "Sample Rate Change" {
    ///             let frequency = tag.data.as_float()
    ///                 .expect("sample rate change should have float data");
    ///             channel.set_frequency(frequency as f32)?;
    ///         }
    ///     }
    /// }
    /// # Ok::<(), fmod::Error>(())
    /// ```
    // XXX: Is the lifetime of the returned tag until the sound is released?
    //     Or is it just until the next call to `get_tag` (needs to take &mut)?
    pub fn get_tag(&self, name: Option<&CStr8>, index: i32) -> Result<Tag<'_>> {
        let mut tag: FMOD_TAG = unsafe { mem::zeroed() };
        ffi!(FMOD_Sound_GetTag(
            self.as_raw(),
            name.map_or(ptr::null(), |name| name.as_c_str().as_ptr()),
            index,
            &mut tag,
        ))?;
        Ok(unsafe { Tag::from_raw(tag)? })
    }
}

/// Format information about a [Sound].
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct SoundFormatInfo {
    /// Type of sound.
    kind: SoundType,
    /// Format of the sound.
    format: SoundFormat,
    /// Number of channels.
    channels: i32,
    /// Number of bits per sample, corresponding to `format`.
    bits_per_sample: i32,
}

/// # Default when played.
impl Sound {
    /// Sets the angles and attenuation levels of a 3D cone shape,
    /// for simulated occlusion which is based on direction.
    ///
    /// When [ChannelControl::set_3d_cone_orientation] is used and a 3D 'cone'
    /// is set up, attenuation will automatically occur for a sound based on the
    /// relative angle of the direction the cone is facing, vs the angle between
    /// the sound and the listener.
    pub fn set_3d_cone_settings(
        &self,
        inside_angle: f32,
        outside_angle: f32,
        outside_volume: f32,
    ) -> Result<()> {
        ffi!(FMOD_Sound_Set3DConeSettings(
            self.as_raw(),
            inside_angle,
            outside_angle,
            outside_volume,
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
    pub fn set_3d_min_max_distance(&self, distance: Range<f32>) -> Result {
        ffi!(FMOD_Sound_Set3DMinMaxDistance(
            self.as_raw(),
            distance.start,
            distance.end,
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
    pub fn set_loop_points(
        &self,
        loop_points: impl RangeBounds<u32>,
        length_type: TimeUnit,
    ) -> Result {
        let loop_start = match loop_points.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let (loop_end, loop_end_type) = match loop_points.end_bound() {
            Bound::Included(&end) => (end, length_type),
            Bound::Excluded(&end) => (end.saturating_sub(1), length_type),
            Bound::Unbounded => (
                self.get_length(TimeUnit::Pcm)?.saturating_sub(1),
                TimeUnit::Pcm,
            ),
        };
        ffi!(FMOD_Sound_SetLoopPoints(
            self.as_raw(),
            loop_start,
            length_type.into_raw(),
            loop_end,
            loop_end_type.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the loop points for a sound.
    pub fn get_loop_points(&self, length_type: TimeUnit) -> Result<RangeInclusive<u32>> {
        let mut start = 0;
        let mut end = 0;
        ffi!(FMOD_Sound_GetLoopPoints(
            self.as_raw(),
            &mut start,
            length_type.into_raw(),
            &mut end,
            length_type.into_raw(),
        ))?;
        Ok(start..=end)
    }
}

/// Standard sound manipulation functions.
impl Sound {
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
