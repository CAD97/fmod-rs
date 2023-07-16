use {
    crate::utils::{fmod_get_string, string_from_utf16be_lossy, string_from_utf16le_lossy},
    fmod::{raw::*, *},
    std::{
        borrow::Cow,
        ffi::CStr,
        mem,
        mem::ManuallyDrop,
        ops::{Bound, Range, RangeBounds, RangeInclusive},
        ptr, slice,
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
    pub fn get_length(&self, unit: TimeUnit) -> Result<u32> {
        let mut length = 0;
        ffi!(FMOD_Sound_GetLength(
            self.as_raw(),
            &mut length,
            unit.into_raw(),
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

enum_struct! {
    /// Recognized audio formats that can be loaded into a Sound.
    pub enum SoundType: FMOD_SOUND_TYPE {
        #[default]
        /// Unknown or custom codec plugin.
        Unknown         = FMOD_SOUND_TYPE_UNKNOWN,
        /// Audio Interchange File Format (.aif, .aiff). Uncompressed integer formats only.
        Aiff            = FMOD_SOUND_TYPE_AIFF,
        /// Microsoft Advanced Systems Format (.asf, .wma, .wmv). Platform provided decoder, available only on Windows.
        Asf             = FMOD_SOUND_TYPE_ASF,
        /// Downloadable Sound (.dls). Multi-sound bank format used by MIDI (.mid).
        Dls             = FMOD_SOUND_TYPE_DLS,
        /// Free Lossless Audio Codec (.flac).
        Flac            = FMOD_SOUND_TYPE_FLAC,
        /// FMOD Sample Bank (.fsb). Proprietary multi-sound bank format. Supported encodings: PCM16, FADPCM, Vorbis, AT9, XMA, Opus.
        Fsb             = FMOD_SOUND_TYPE_FSB,
        /// Impulse Tracker (.it).
        It              = FMOD_SOUND_TYPE_IT,
        /// Musical Instrument Digital Interface (.mid).
        Midi            = FMOD_SOUND_TYPE_MIDI,
        /// Protracker / Fasttracker Module File (.mod).
        Mod             = FMOD_SOUND_TYPE_MOD,
        /// Moving Picture Experts Group (.mp2, .mp3). Also supports .wav (RIFF) container format.
        Mpeg            = FMOD_SOUND_TYPE_MPEG,
        /// Ogg Vorbis (.ogg).
        OggVorbis       = FMOD_SOUND_TYPE_OGGVORBIS,
        /// Play list information container (.asx, .pls, .m3u, .wax). No audio, tags only.
        Playlist        = FMOD_SOUND_TYPE_PLAYLIST,
        /// Raw uncompressed PCM data (.raw).
        Raw             = FMOD_SOUND_TYPE_RAW,
        /// ScreamTracker 3 Module (.s3m).
        S3m             = FMOD_SOUND_TYPE_S3M,
        /// User created sound.
        User            = FMOD_SOUND_TYPE_USER,
        /// Microsoft Waveform Audio File Format (.wav). Supported encodings: Uncompressed PCM, IMA ADPCM. Platform provided ACM decoder extensions, available only on Windows.
        Wav             = FMOD_SOUND_TYPE_WAV,
        /// FastTracker 2 Extended Module (.xm).
        Xm              = FMOD_SOUND_TYPE_XM,
        /// Microsoft XMA bit-stream supported by FSB (.fsb) container format. Platform provided decoder, available only on Xbox.
        Xma             = FMOD_SOUND_TYPE_XMA,
        /// Apple Audio Queue decoder (.mp4, .m4a, .mp3). Supported encodings: AAC, ALAC, MP3. Platform provided decoder, available only on iOS / tvOS devices.
        AudioQueue      = FMOD_SOUND_TYPE_AUDIOQUEUE,
        /// Sony ATRAC9 bit-stream supported by FSB (.fsb) container format. Platform provided decoder, available only on PlayStation.
        At9             = FMOD_SOUND_TYPE_AT9,
        /// Vorbis bit-stream supported by FSB (.fsb) container format.
        Vorbis          = FMOD_SOUND_TYPE_VORBIS,
        /// Microsoft Media Foundation decoder (.asf, .wma, .wmv, .mp4, .m4a). Platform provided decoder, available only on UWP.
        MediaFoundation = FMOD_SOUND_TYPE_MEDIA_FOUNDATION,
        /// Google Media Codec decoder (.m4a, .mp4). Platform provided decoder, available only on Android.
        MediaCodec      = FMOD_SOUND_TYPE_MEDIACODEC,
        /// FMOD Adaptive Differential Pulse Code Modulation bit-stream supported by FSB (.fsb) container format.
        Fadpcm          = FMOD_SOUND_TYPE_FADPCM,
        /// Opus bit-stream supported by FSB (.fsb) container format. Platform provided decoder, available only on Xbox Series X|S.
        Opus            = FMOD_SOUND_TYPE_OPUS,
    }

    /// These definitions describe the native format of the hardware or software buffer that will be used.
    pub enum SoundFormat: FMOD_SOUND_FORMAT {
        #[default]
        /// Uninitalized / unknown.
        None      = FMOD_SOUND_FORMAT_NONE,
        /// 8bit integer PCM data.
        Pcm8      = FMOD_SOUND_FORMAT_PCM8,
        /// 16bit integer PCM data.
        Pcm16     = FMOD_SOUND_FORMAT_PCM16,
        /// 24bit integer PCM data.
        Pcm24     = FMOD_SOUND_FORMAT_PCM24,
        /// 32bit integer PCM data.
        Pcm32     = FMOD_SOUND_FORMAT_PCM32,
        /// 32bit floating point PCM data.
        PcmFloat  = FMOD_SOUND_FORMAT_PCMFLOAT,
        /// Sound data is in its native compressed format. See [Mode::CreateCompressedSample]
        Bitstream = FMOD_SOUND_FORMAT_BITSTREAM,
    }

    /// List of tag data / metadata types that could be stored within a sound. These include id3 tags, metadata from netstreams and vorbis/asf data.
    pub enum TagType: FMOD_TAGTYPE {
        /// Tag type that is not recognized by FMOD
        Unknown       = FMOD_TAGTYPE_UNKNOWN,
        /// MP3 ID3 Tag 1.0. Typically 1 tag stored 128 bytes from end of an MP3 file.
        Id3v1         = FMOD_TAGTYPE_ID3V1,
        /// MP3 ID3 Tag 2.0. Variable length tags with more than 1 possible.
        Id3v2         = FMOD_TAGTYPE_ID3V2,
        /// Metadata container used in Vorbis, FLAC, Theora, Speex and Opus file formats.
        VorbisComment = FMOD_TAGTYPE_VORBISCOMMENT,
        /// SHOUTcast internet stream metadata which can be issued during playback.
        ShoutCast     = FMOD_TAGTYPE_SHOUTCAST,
        /// Icecast internet stream metadata which can be issued during playback.
        Icecast       = FMOD_TAGTYPE_ICECAST,
        /// Advanced Systems Format metadata typically associated with Windows Media formats such as WMA.
        Asf           = FMOD_TAGTYPE_ASF,
        /// Metadata stored inside a MIDI file.
        ///
        /// Remarks. A midi file contains 16 channels. Not all of them are used, or in order. Use the tag 'Channel mask' and 'Number of channels' to find the channels used, to use with [Sound::set_music_channel_volume] / [Sound::get_music_channel_volume]. For example if the mask is 1001b, there are 2 channels, and channel 0 and channel 3 are the 2 channels used with the above functions.
        Midi          = FMOD_TAGTYPE_MIDI,
        /// Playlist files such as PLS,M3U,ASX and WAX will populate playlist information through this tag type.
        Playlist      = FMOD_TAGTYPE_PLAYLIST,
        /// Tag type used by FMOD's MIDI, MOD, S3M, XM, IT format support, and netstreams to notify of internet stream events like a sample rate change.
        Fmod          = FMOD_TAGTYPE_FMOD,
        /// For codec developers, this tag type can be used with [CodecMetadataFunc] to generate custom metadata.
        User          = FMOD_TAGTYPE_USER,
    }
}

raw! {
    enum_struct! {
        /// List of tag data / metadata types.
        ///
        /// See [Tag] structure for tag length in bytes.
        pub enum TagDataType: FMOD_TAGDATATYPE {
            /// Raw binary data. see [Tag] structure for length of data in bytes.
            Binary        = FMOD_TAGDATATYPE_BINARY,
            /// Integer - Note this integer could be 8bit / 16bit / 32bit / 64bit. See [Tag] structure for integer size (1 vs 2 vs 4 vs 8 bytes).
            Int           = FMOD_TAGDATATYPE_INT,
            /// IEEE floating point number. See [Tag] structure to confirm if the float data is 32bit or 64bit (4 vs 8 bytes).
            Float         = FMOD_TAGDATATYPE_FLOAT,
            /// 8bit ASCII char string. See [Tag] structure for string length in bytes.
            String        = FMOD_TAGDATATYPE_STRING,
            /// 16bit UTF string. Assume little endian byte order. See [Tag] structure for string length in bytes.
            StringUtf16   = FMOD_TAGDATATYPE_STRING_UTF16,
            /// 16bit UTF string Big endian byte order. See [Tag] structure for string length in bytes.
            StringUtf16be = FMOD_TAGDATATYPE_STRING_UTF16BE,
            /// 8 bit UTF string. See [Tag] structure for string length in bytes.
            StringUtf8    = FMOD_TAGDATATYPE_STRING_UTF8,
        }
    }
}

/// Tag data / metadata description.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag<'a> {
    /// Tag type.
    pub kind: TagType,
    /// Name.
    pub name: Cow<'a, str>,
    /// Tag data.
    pub data: TagData<'a>,
    /// True if this tag has been updated since last being accessed with [Sound::get_tag]
    pub updated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagData<'a> {
    Binary(&'a [u8]),
    Int(i64),
    Float(f64),
    Str(Cow<'a, str>),
}

impl Tag<'_> {
    raw! {
        pub unsafe fn from_raw(tag: FMOD_TAG) -> Result<Self> {
            let name = CStr::from_ptr(tag.name);
            let name = name.to_string_lossy();
            let data = slice::from_raw_parts(tag.data as *const u8, ix!(tag.datalen));
            let data = match TagDataType::from_raw(tag.datatype) {
                TagDataType::Binary => TagData::Binary(data),
                TagDataType::Int if data.len() == 1 => TagData::Int((tag.data as *const u8).read() as _),
                TagDataType::Int if data.len() == 2 => TagData::Int((tag.data as *const u16).read_unaligned() as _),
                TagDataType::Int if data.len() == 4 => TagData::Int((tag.data as *const u32).read_unaligned() as _),
                TagDataType::Int if data.len() == 8 => TagData::Int((tag.data as *const u64).read_unaligned() as _),
                TagDataType::Float if data.len() == 4 => TagData::Float((tag.data as *const f32).read_unaligned() as _),
                TagDataType::Float if data.len() == 8 => TagData::Float((tag.data as *const f64).read_unaligned() as _),
                TagDataType::String | TagDataType::StringUtf8 => TagData::Str(String::from_utf8_lossy(data)),
                TagDataType::StringUtf16 => TagData::Str(Cow::Owned(string_from_utf16le_lossy(data))),
                TagDataType::StringUtf16be => TagData::Str(Cow::Owned(string_from_utf16be_lossy(data))),
                r#type => {
                    whoops!(no_panic: "unknown {type:?} (len {}) encountered", tag.datalen);
                    yeet!(Error::RustPanicked);
                },
            };
            Ok(Tag {
                kind: TagType::from_raw(tag.r#type),
                name,
                data,
                updated: tag.updated == 0,
            })
        }
    }
}

impl<'a> TagData<'a> {
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            TagData::Binary(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            TagData::Int(data) => Some(*data),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            TagData::Float(data) => Some(*data),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            TagData::Str(data) => Some(data),
            _ => None,
        }
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
        Ok(unsafe { Sound::from_raw_opt(parent_sound) })
    }
}

/// # Data reading.
impl Sound {
    /// Retrieves the state a sound is in after being opened with the non
    /// blocking flag, or the current state of the streaming buffer.
    ///
    /// When a sound is opened with [`Mode::NonBlocking`], it is opened and
    /// prepared in the background, or asynchronously. This allows the main
    /// application to execute without stalling on audio loads. This function
    /// will describe the state of the asynchronous load routine i.e. whether
    /// it has succeeded, failed or is still in progress.
    ///
    /// **Note:** Always check the return value to determine the state of the
    /// sound. Do not assume that if this function returns `Ok` then the sound
    /// has finished loading.
    pub fn get_open_state(&self) -> Result<OpenState> {
        let mut state = OpenState::zeroed();
        ffi!(FMOD_Sound_GetOpenState(
            self.as_raw(),
            state.as_raw_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        ))?;
        Ok(state)
    }

    /// Retrieves the state a sound is in after being opened with the non
    /// blocking flag, or the current state of the streaming buffer.
    ///
    /// When a sound is opened with [`Mode::NonBlocking`], it is opened and
    /// prepared in the background, or asynchronously. This allows the main
    /// application to execute without stalling on audio loads. This function
    /// will describe the state of the asynchronous load routine i.e. whether
    /// it has succeeded, failed or is still in progress.
    ///
    /// If `starving` is true, then you will most likely hear a
    /// stuttering/repeating sound as the decode buffer loops on itself and
    /// replays old data. With the ability to detect stream starvation, muting
    /// the sound with [`ChannelControl::set_mute`] will keep the stream quiet
    /// until it is not starving any more.
    ///
    /// **Note:** Always check the return value to determine the state of the
    /// sound. Do not assume that if this function returns `Ok` then the sound
    /// has finished loading.
    pub fn get_open_state_info(&self) -> Result<OpenStateInfo> {
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
        Ok(OpenStateInfo {
            state,
            percent_buffered,
            starving: starving != 0,
            disk_busy: disk_busy != 0,
        })
    }

    /// Reads data from an opened sound to a specified buffer,
    /// using FMOD's internal codecs.
    ///
    /// This can be used for decoding data offline in small pieces (or big
    /// pieces), rather than playing and capturing it, or loading the whole file
    /// at once and having to [`Sound::lock`] / [`Sound::unlock`] the data.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">🦀</span><span>
    /// FMOD.rs returns `Ok(0)` on EOF, matching the [`Read`](io::Read) trait,
    /// whereas raw FMOD returns `Error::FileEof`.
    /// </span></div></div>
    ///
    /// As a non streaming sound reads and decodes the whole file then closes it
    /// upon calling [`System::create_sound`], [`Sound::read_data`] will then
    /// not work because the file handle is closed. Use [`Mode::OpenOnly`] to
    /// stop FMOD reading/decoding the file. If [`Mode::OpenOnly`] flag is used
    /// when opening a sound, it will leave the file handle open, and FMOD will
    /// not read/decode any data internally, so the read cursor will stay at
    /// position 0. This will allow the user to read the data from the start.
    ///
    /// For streams, the streaming engine will decode a small chunk of data and
    /// this will advance the read cursor. You need to either use
    /// [`Mode::OpenOnly`] to stop the stream pre-buffering or call
    /// [`Sound::seek_data`] to reset the read cursor back to the start of the
    /// file, otherwise it will appear as if the start of the stream is missing.
    /// [`Channel::set_position`] will have the same result. These functions
    /// will flush the stream buffer and read in a chunk of audio internally.
    /// This is why if you want to read from an absolute position you should use
    /// [`Sound::seek_data`] and not the previously mentioned functions.
    ///
    /// If you are calling [`Sound::read_data`] and [`Sound::seek_data`] on a
    /// stream, information functions such as [`Channel::get_position`] may give
    /// misleading results. Calling [`Channel::set_position`] will cause the
    /// streaming engine to reset and flush the stream, leading to the time
    /// values returning to their correct position.
    pub fn read_data(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut read = 0;
        ffi!(FMOD_Sound_ReadData(
            self.as_raw(),
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
            &mut read,
        ))
        .or_else(|e| if e == Error::FileEof { Ok(()) } else { Err(e) })?;
        Ok(ix!(read))
    }

    /// Seeks a sound for use with data reading, using FMOD's internal codecs.
    ///
    /// For use in conjunction with [`Sound::read_data`] and [`Mode::OpenOnly`].
    ///
    /// For streaming sounds, if this function is called, it will advance the
    /// internal file pointer but not update the streaming engine. This can lead
    /// to de-synchronization of position information for the stream and audible
    /// playback.
    ///
    /// A stream can reset its stream buffer and position synchronization by
    /// calling [`Channel::set_position`]. This causes reset and flush of the
    /// stream buffer.
    pub fn seek_data(&self, pcm: u32) -> Result {
        ffi!(FMOD_Sound_SeekData(self.as_raw(), pcm))?;
        Ok(())
    }

    /// Gives access to a portion or all the sample data of a sound for direct manipulation.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">🦀</span><span>
    /// Dropping `SoundReadGuard` will call `Sound::unlock` and unlock the data.
    /// </span></div></div>
    ///
    /// With this function you get access to the raw audio data. If the data is
    /// 8, 16, 24 or 32bit PCM data, mono or stereo data, you must take this
    /// into consideration when processing the data. See [Sample Data] for more
    /// information.
    ///
    /// [Sample Data]: https://fmod.com/docs/2.02/api/glossary.html#sample-data
    ///
    /// If the sound is created with [`Mode::CreateCompressedSample`] the data
    /// retrieved will be the compressed bitstream.
    ///
    /// It is not possible to lock the following:
    ///
    /// - A parent sound containing subsounds. A parent sound has no audio data
    ///   and [`Error::SubSounds`] will be returned.
    /// - A stream / sound created with [`Mode::CreateStream`].
    ///   [`Error::BadCommand`] will be returned in this case.
    ///
    /// The names `lock`/`unlock` are a legacy reference to older Operating
    /// System APIs that used to cause a mutex lock on the data, so that it
    /// could not be written to while the 'lock' was in place. This is no
    /// longer the case with FMOD and data can be 'locked' multiple times
    /// from different places/threads at once.
    pub fn lock(&self, offset: u32, length: u32) -> Result<SampleDataLock<'_>> {
        let mut ptr1 = ptr::null_mut();
        let mut ptr2 = ptr::null_mut();
        let mut len1 = 0;
        let mut len2 = 0;
        ffi!(FMOD_Sound_Lock(
            self.as_raw(),
            offset,
            length,
            &mut ptr1,
            &mut ptr2,
            &mut len1,
            &mut len2,
        ))?;
        unsafe {
            Ok(SampleDataLock {
                sound: self,
                part1: slice::from_raw_parts(ptr1.cast(), ix!(len1)),
                part2: if !ptr2.is_null() {
                    slice::from_raw_parts(ptr2.cast(), ix!(len2))
                } else {
                    slice::from_raw_parts(ptr1.cast::<u8>().add(ix!(len1)), 0)
                },
            })
        }
    }

    /// Finalizes a previous sample data lock and submits it back to the Sound
    /// object.
    ///
    /// If an unlock is not performed on PCM data, then sample loops may produce
    /// audible clicks.
    ///
    /// The names `lock`/`unlock` are a legacy reference to older Operating
    /// System APIs that used to cause a mutex lock on the data, so that it
    /// could not be written to while the 'lock' was in place. This is no
    /// longer the case with FMOD and data can be 'locked' multiple times
    /// from different places/threads at once.
    ///
    /// # Safety
    ///
    /// The locked slices must have been obtained from a previous matched call
    /// to [`Sound::lock`].
    pub unsafe fn unlock(&self, part1: &[u8], part2: &[u8]) -> Result {
        ffi!(FMOD_Sound_Unlock(
            self.as_raw(),
            part1.as_ptr().cast_mut().cast(),
            part2.as_ptr().cast_mut().cast(),
            part1.len() as u32,
            part2.len() as u32,
        ))?;
        Ok(())
    }
}

enum_struct! {
    /// These values describe what state a sound is in after [Mode::NonBlocking] has been used to open it.
    ///
    /// With streams, if you are using [Mode::NonBlocking], note that if the user calls [Sound::get_sub_sound], a stream will go into [OpenState::Seeking] state and sound related commands will return [Error::NotReady].
    ///
    /// With streams, if you are using [Mode::NonBlocking], note that if the user calls [Channel::get_position], a stream will go into [OpenState::SetPosition] state and sound related commands will return [Error::NotReady].
    pub enum OpenState: FMOD_OPENSTATE {
        /// Opened and ready to play.
        Ready       = FMOD_OPENSTATE_READY,
        /// Initial load in progress.
        Loading     = FMOD_OPENSTATE_LOADING,
        /// Failed to open - file not found, out of memory etc. See return value of [Sound::get_open_state] for what happened.
        Error       = FMOD_OPENSTATE_ERROR,
        /// Connecting to remote host (internet sounds only).
        Connecting  = FMOD_OPENSTATE_CONNECTING,
        /// Buffering data.
        Buffering   = FMOD_OPENSTATE_BUFFERING,
        /// Seeking to subsound and re-flushing stream buffer.
        Seeking     = FMOD_OPENSTATE_SEEKING,
        /// Ready and playing, but not possible to release at this time without stalling the main thread.
        Playing     = FMOD_OPENSTATE_PLAYING,
        /// Seeking within a stream to a different position.
        SetPosition = FMOD_OPENSTATE_SETPOSITION,
    }
}

// XXX: io::Read and io::Seek impls?

/// A read lock on a sound's sample data.
#[derive(Debug, Clone)]
pub struct SampleDataLock<'a> {
    sound: &'a Sound,
    part1: &'a [u8],
    part2: &'a [u8],
}

impl SampleDataLock<'_> {
    /// Returns the locked sample data.
    ///
    /// The first slice borrows from the sample buffer directly. If the locked
    /// data exceeds the length of the sample buffer, the second slice holds
    /// any excess data.
    pub fn get(&self) -> (&[u8], &[u8]) {
        (self.part1, self.part2)
    }

    /// Finalizes the sample data lock and submits it back to the Sound object.
    pub fn unlock(self) -> Result {
        let this = ManuallyDrop::new(self);
        unsafe { this.sound.unlock(this.part1, this.part2) }
    }
}

impl<'a> Drop for SampleDataLock<'a> {
    fn drop(&mut self) {
        match unsafe { self.sound.unlock(self.part1, self.part2) } {
            Ok(()) => (),
            Err(e) => whoops!("failed to unlock sound: {e}"),
        }
    }
}

/// The state a sound is in after being opened with the non blocking flag,
/// or the current state of the streaming buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct OpenStateInfo {
    /// Open state of a sound.
    pub state: OpenState,
    /// Filled percentage of a stream's file buffer.
    pub percent_buffered: u32,
    /// Starving state. `true` if a stream has decoded
    /// more than the stream file buffer has ready.
    pub starving: bool,
    /// Disk is currently being accessed for this sound.
    pub disk_busy: bool,
}

/// # Music.
impl Sound {
    /// Gets the number of music channels inside a MOD/S3M/XM/IT/MIDI file.
    pub fn get_music_num_channels(&self) -> Result<i32> {
        let mut channels = 0;
        ffi!(FMOD_Sound_GetMusicNumChannels(self.as_raw(), &mut channels))?;
        Ok(channels)
    }

    /// Sets the volume of a MOD/S3M/XM/IT/MIDI music channel volume.
    pub fn set_music_channel_volume(&self, channel: i32, volume: f32) -> Result {
        ffi!(FMOD_Sound_SetMusicChannelVolume(
            self.as_raw(),
            channel,
            volume,
        ))?;
        Ok(())
    }

    /// Retrieves the volume of a MOD/S3M/XM/IT/MIDI music channel volume.
    pub fn get_music_channel_volume(&self, channel: i32) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_Sound_GetMusicChannelVolume(
            self.as_raw(),
            channel,
            &mut volume,
        ))?;
        Ok(volume)
    }

    /// Sets the relative speed of MOD/S3M/XM/IT/MIDI music.
    pub fn set_music_speed(&self, speed: f32) -> Result {
        ffi!(FMOD_Sound_SetMusicSpeed(self.as_raw(), speed))?;
        Ok(())
    }

    /// Retrieves the relative speed of MOD/S3M/XM/IT/MIDI music.
    pub fn get_music_speed(&self) -> Result<f32> {
        let mut speed = 0.0;
        ffi!(FMOD_Sound_GetMusicSpeed(self.as_raw(), &mut speed))?;
        Ok(speed)
    }
}

/// # Synchronization / markers.
impl Sound {
    /// Retrieve a sync point.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn get_sync_point(&self, index: i32) -> Result<&SyncPoint> {
        let mut point = ptr::null_mut();
        ffi!(FMOD_Sound_GetSyncPoint(self.as_raw(), index, &mut point))?;
        Ok(unsafe { SyncPoint::from_raw(point) })
    }

    /// Retrieves information on an embedded sync point.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn get_sync_point_name(&self, sync_point: &SyncPoint, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_Sound_GetSyncPointInfo(
                    self.as_raw(),
                    sync_point.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as i32,
                    ptr::null_mut(),
                    TimeUnit::zeroed().into_raw(),
                ))
            })?;
        }
        Ok(())
    }

    /// Retrieves information on an embedded sync point.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn get_sync_point_offset(&self, sync_point: &SyncPoint, unit: TimeUnit) -> Result<u32> {
        let mut offset = 0;
        ffi!(FMOD_Sound_GetSyncPointInfo(
            self.as_raw(),
            sync_point.as_raw(),
            ptr::null_mut(),
            0,
            &mut offset,
            unit.into_raw(),
        ))?;
        Ok(offset)
    }

    /// Adds a sync point at a specific time within the sound.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn add_sync_point(
        &self,
        offset: Time,
        name: Option<&CStr8>,
    ) -> Result<Handle<'static, SyncPoint>> {
        let mut point = ptr::null_mut();
        ffi!(FMOD_Sound_AddSyncPoint(
            self.as_raw(),
            offset.value,
            offset.unit.into_raw(),
            name.map(|s| s.as_c_str().as_ptr()).unwrap_or(ptr::null()),
            &mut point,
        ))?;
        Ok(unsafe { Handle::from_raw(point) })
    }

    /// Deletes a sync point within the sound.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn delete_sync_point(&self, sync_point: Handle<'_, SyncPoint>) -> Result {
        ffi!(FMOD_Sound_DeleteSyncPoint(
            self.as_raw(),
            Handle::into_raw(sync_point),
        ))?;
        Ok(())
    }
}

/// # General.
impl Sound {
    // TODO: set_user_data, get_user_data

    /// Retrieves the parent System object.
    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_Sound_GetSystemObject(self.as_raw(), &mut system))?;
        Ok(unsafe { System::from_raw(system) })
    }
}