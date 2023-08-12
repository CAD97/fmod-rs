use {
    crate::utils::{fmod_get_string, string_from_utf16be_lossy, string_from_utf16le_lossy},
    fmod::{raw::*, *},
    std::{borrow::Cow, ffi::CStr, mem, ptr, slice},
};

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

/// Tag data.
#[derive(Debug, Clone, PartialEq)]
pub enum TagData<'a> {
    /// Raw binary data.
    Binary(&'a [u8]),
    /// Integer data.
    Int(i64),
    /// Floating point data.
    Float(f64),
    /// String data.
    Str(Cow<'a, str>),
}

#[allow(missing_docs)]
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

fmod_enum! {
    /// Recognized audio formats that can be loaded into a Sound.
    #[derive(Default)]
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
}

fmod_enum! {
    /// These definitions describe the native format of the hardware or software buffer that will be used.
    #[derive(Default)]
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
}

fmod_enum! {
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
    fmod_enum! {
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

/// Format information about a [Sound].
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub struct SoundFormatInfo {
    /// Type of sound.
    pub kind: SoundType,
    /// Format of the sound.
    pub format: SoundFormat,
    /// Number of channels.
    pub channels: i32,
    /// Number of bits per sample, corresponding to `format`.
    pub bits_per_sample: i32,
}
