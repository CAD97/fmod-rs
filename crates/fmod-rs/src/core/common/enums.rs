use fmod::{raw::*, *};

raw! {
    enum_struct! {
        /// Identifier used to distinguish between Channel and ChannelGroup in the ChannelControl callback.
        pub enum ChannelControlType: FMOD_CHANNELCONTROL_TYPE {
            /// Type representing [Channel]
            Channel      = FMOD_CHANNELCONTROL_CHANNEL,
            /// Type representing [ChannelGroup]
            ChannelGroup = FMOD_CHANNELCONTROL_CHANNELGROUP,
        }
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

    /// Values specifying behavior when a sound group's max audible value is exceeded.
    ///
    /// When using [SoundGroupBehavior::Mute], [SoundGroup::set_mute_fade_speed] can be used to stop a sudden transition.
    /// Instead, the time specified will be used to cross fade between the sounds that go silent and the ones that become audible.
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

raw! {
    enum_struct! {
        /// Types of callbacks called by Channels and ChannelGroups.
        pub enum ChannelControlCallbackType: FMOD_CHANNELCONTROL_CALLBACK_TYPE {
            /// Called when a sound ends. Supported by [Channel] only.
            End          = FMOD_CHANNELCONTROL_CALLBACK_END,
            /// Called when a [Channel] is made virtual or real. Supported by [Channel] objects only.
            ///
            /// - `command_data_1`: (int) 0 represents 'virtual to real' and 1 represents 'real to virtual'.
            VirtualVoice = FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE,
            /// Called when a syncpoint is encountered. Can be from wav file markers or user added. Supported by [Channel] only.
            ///
            /// - `command_data_1`: (int) representing the index of the sync point for use with [Sound::get_sync_point_info].
            SyncPoint    = FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT,
            /// Called when geometry occlusion values are calculated. Can be used to clamp or change the value. Supported by [Channel] and [ChannelGroup].
            Occlusion    = FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION,
        }
    }
}

raw! {
    enum_struct! {
        /// References to built in DSP positions that reside in a Channel or ChannelGroup DSP chain.
        ///
        /// Before any [Dsp]s have been added by the user, there is only one [Dsp] available for a [Channel] or [ChannelGroup]. This is of type [DspType::Fader]. This handles volume and panning for a [Channel] or [ChannelGroup].
        /// As only 1 [Dsp] exists by default, initially [ChannelControlDspIndex::Head], [ChannelControlDspIndex::Tail] and [ChannelControlDspIndex::Fader] all reference the same DSP.
        pub enum ChannelControlDspIndex: FMOD_CHANNELCONTROL_DSP_INDEX {
            /// Head of the DSP chain, equivalent of index 0.
            Head  = FMOD_CHANNELCONTROL_DSP_HEAD,
            /// Built in fader DSP.
            Fader = FMOD_CHANNELCONTROL_DSP_FADER,
            /// Tail of the DSP chain, equivalent of the number of [Dsp]s minus 1.
            Tail  = FMOD_CHANNELCONTROL_DSP_TAIL,
        }
    }
}

raw! {
    enum_struct! {
        /// Types of callbacks called by DSPs.
        ///
        /// Callbacks are called from the game thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        pub enum DspCallbackType: FMOD_DSP_CALLBACK_TYPE {
            /// Called when a DSP's data parameter can be released.
            DataParameterInfo = FMOD_DSP_CALLBACK_DATAPARAMETERRELEASE,
        }
    }
}

enum_struct! {
    /// List of connection types between 2 DSP nodes.
    pub enum DspConnectionType: FMOD_DSPCONNECTION_TYPE {
        #[default]
        /// Default connection type. Audio is mixed from the input to the output [Dsp]'s audible buffer.
        ///
        /// Default [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s audible buffer, meaning it will be part of the audible signal. A standard connection will execute its input [Dsp] if it has not been executed before.
        Standard      = FMOD_DSPCONNECTION_TYPE_STANDARD,
        /// Sidechain connection type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer.
        ///
        /// Sidechain [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, meaning it will NOT be part of the audible signal. A sidechain connection will execute its input [Dsp] if it has not been executed before.
        ///
        /// The purpose of the seperate sidechain buffer in a [Dsp], is so that the [Dsp] effect can privately access for analysis purposes. An example of use in this case, could be a compressor which analyzes the signal, to control its own effect parameters (ie a compression level or gain).
        ///
        /// For the effect developer, to accept sidechain data, the sidechain data will appear in the [DspState] struct which is passed into the read callback of a [Dsp] unit.
        ///
        /// [DspState::sidechain_data] and [DspState::sidechain_channels] will hold the mixed result of any sidechain data flowing into it.
        Sidechain     = FMOD_DSPCONNECTION_TYPE_SIDECHAIN,
        /// Send connection type. Audio is mixed from the input to the output [Dsp]'s audible buffer, but the input is NOT executed, only copied from. A standard connection or sidechain needs to make an input execute to generate data.
        ///
        /// Send [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s audible buffer, meaning it will be part of the audible signal. A send connection will NOT execute its input [Dsp] if it has not been executed before.
        ///
        /// A send connection will only read what exists at the input's buffer at the time of executing the output [Dsp] unit (which can be considered the 'return')
        Send          = FMOD_DSPCONNECTION_TYPE_SEND,
        /// Send sidechain connection type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, but the input is NOT executed, only copied from. A standard connection or sidechain needs to make an input execute to generate data.
        ///
        /// Send sidechain [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, meaning it will NOT be part of the audible signal. A send sidechain connection will NOT execute its input [Dsp] if it has not been executed before.
        ///
        /// A send sidechain connection will only read what exists at the input's buffer at the time of executing the output [Dsp] unit (which can be considered the 'sidechain return').
        ///
        /// For the effect developer, to accept sidechain data, the sidechain data will appear in the [DspState] struct which is passed into the read callback of a [Dsp] unit.
        ///
        /// [DspState::sidechain_data] and [DspState::sidechain_channels] will hold the mixed result of any sidechain data flowing into it.
        SendSidechain = FMOD_DSPCONNECTION_TYPE_SEND_SIDECHAIN,
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

enum_struct! {
    #[cfg_attr(feature = "unstable", doc(cfg(target_os = "ios")))]
    /// Control whether the sound will use a the dedicated hardware decoder or a
    /// software codec.
    ///
    /// Every devices has a single hardware decoder and unlimited software
    /// decoders.
    pub enum AudioQueueCodecPolicy: FMOD_AUDIOQUEUE_CODECPOLICY {
        /// Try hardware first, if it's in use or prohibited by audio session,
        /// try software.
        Default      = FMOD_AUDIOQUEUE_CODECPOLICY_DEFAULT,
        /// `kAudioQueueHardwareCodecPolicy_UseSoftwareOnly` ~ try software,
        /// if not available fail.
        SoftwareOnly = FMOD_AUDIOQUEUE_CODECPOLICY_SOFTWAREONLY,
        /// `kAudioQueueHardwareCodecPolicy_UseHardwareOnly` ~ try hardware,
        /// if not available fail.
        HardwareOnly = FMOD_AUDIOQUEUE_CODECPOLICY_HARDWAREONLY,
    }
}
