use fmod::raw::*;

macro_rules! enum_struct {
    {$(
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Repr:ty {$(
            $(#[$vmeta:meta])*
            $Variant:ident = $value:expr,
        )*}
    )*} => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis struct $Name {
            raw: $Repr,
        }

        impl $Name {
            raw! {
                #[allow(clippy::missing_safety_doc)]
                pub const unsafe fn from_raw(raw: $Repr) -> Self {
                    Self { raw }
                }

                pub const fn into_raw(self) -> $Repr {
                    self.raw
                }
            }

            $(
                $(#[$vmeta])*
                #[allow(non_upper_case_globals)]
                pub const $Variant: Self = unsafe { Self::from_raw($value) };
            )*
        }
    )*};
}

enum_struct! {
    pub enum DriverState: u32 {
        Connected = FMOD_DRIVER_STATE_CONNECTED,
        Default   = FMOD_DRIVER_STATE_DEFAULT,
    }

    pub enum TimeUnit: u32 {
        Ms              = FMOD_TIMEUNIT_MS,
        Pcm             = FMOD_TIMEUNIT_PCM,
        PcmBytes        = FMOD_TIMEUNIT_PCMBYTES,
        RawBytes        = FMOD_TIMEUNIT_RAWBYTES,
        PcmFraction     = FMOD_TIMEUNIT_PCMFRACTION,
        ModOrder        = FMOD_TIMEUNIT_MODORDER,
        ModRow          = FMOD_TIMEUNIT_MODROW,
        ModPattern      = FMOD_TIMEUNIT_MODPATTERN,
    }

    pub enum Mode: u32 {
        Default                 = FMOD_DEFAULT,
        LoopOff                 = FMOD_LOOP_OFF,
        LoopNormal              = FMOD_LOOP_NORMAL,
        LoopBidi                = FMOD_LOOP_BIDI,
        D2                      = FMOD_2D,
        D3                      = FMOD_3D,
        CreateStream            = FMOD_CREATESTREAM,
        CreateSample            = FMOD_CREATESAMPLE,
        CreateCompressedSample  = FMOD_CREATECOMPRESSEDSAMPLE,
        OpenUser                = FMOD_OPENUSER,
        OpenMemory              = FMOD_OPENMEMORY,
        OpenMemoryPoint         = FMOD_OPENMEMORY_POINT,
        OpenRaw                 = FMOD_OPENRAW,
        OpenOnly                = FMOD_OPENONLY,
        AccurateTime            = FMOD_ACCURATETIME,
        MpegSearch              = FMOD_MPEGSEARCH,
        NonBlocking             = FMOD_NONBLOCKING,
        Unique                  = FMOD_UNIQUE,
        HeadRelative3d          = FMOD_3D_HEADRELATIVE,
        WorldRelative3d         = FMOD_3D_WORLDRELATIVE,
        InverseRolloff3d        = FMOD_3D_INVERSEROLLOFF,
        LinearRolloff3d         = FMOD_3D_LINEARROLLOFF,
        LinearSquareRolloff3d   = FMOD_3D_LINEARSQUAREROLLOFF,
        InverseTaperedRolloff3d = FMOD_3D_INVERSETAPEREDROLLOFF,
        CustomRolloff3d         = FMOD_3D_CUSTOMROLLOFF,
        IgnoreGeometry3d        = FMOD_3D_IGNOREGEOMETRY,
        IgnoreTags              = FMOD_IGNORETAGS,
        LowMem                  = FMOD_LOWMEM,
        VirtualPlayFromStart    = FMOD_VIRTUAL_PLAYFROMSTART,
    }

    pub enum ThreadPriority: i32 {
        // Platform specific priority range
        PlatformMin = FMOD_THREAD_PRIORITY_PLATFORM_MIN,
        PlatformMax = FMOD_THREAD_PRIORITY_PLATFORM_MAX as i32,
        // Platform agnostic priorities, maps internall to platform specific value
        Default  = FMOD_THREAD_PRIORITY_DEFAULT,
        Low      = FMOD_THREAD_PRIORITY_LOW,
        Medium   = FMOD_THREAD_PRIORITY_MEDIUM,
        High     = FMOD_THREAD_PRIORITY_HIGH,
        VeryHigh = FMOD_THREAD_PRIORITY_VERY_HIGH,
        Extreme  = FMOD_THREAD_PRIORITY_EXTREME,
        Critical = FMOD_THREAD_PRIORITY_CRITICAL,
        // Thread defaults
        Mixer            = FMOD_THREAD_PRIORITY_MIXER,
        Feeder           = FMOD_THREAD_PRIORITY_FEEDER,
        Stream           = FMOD_THREAD_PRIORITY_STREAM,
        File             = FMOD_THREAD_PRIORITY_FILE,
        NonBlocking      = FMOD_THREAD_PRIORITY_NONBLOCKING,
        Record           = FMOD_THREAD_PRIORITY_RECORD,
        Geometry         = FMOD_THREAD_PRIORITY_GEOMETRY,
        Profiler         = FMOD_THREAD_PRIORITY_PROFILER,
        StudioUpdate     = FMOD_THREAD_PRIORITY_STUDIO_UPDATE,
        StudioLoadBank   = FMOD_THREAD_PRIORITY_STUDIO_LOAD_BANK,
        StudioLoadSample = FMOD_THREAD_PRIORITY_STUDIO_LOAD_SAMPLE,
        Convolution1     = FMOD_THREAD_PRIORITY_CONVOLUTION1,
        Convolution2     = FMOD_THREAD_PRIORITY_CONVOLUTION2,
    }

    pub enum StackSize: u32 {
        Default          = FMOD_THREAD_STACK_SIZE_DEFAULT,
        Mixer            = FMOD_THREAD_STACK_SIZE_MIXER,
        Feeder           = FMOD_THREAD_STACK_SIZE_FEEDER,
        Stream           = FMOD_THREAD_STACK_SIZE_STREAM,
        File             = FMOD_THREAD_STACK_SIZE_FILE,
        NonBlocking      = FMOD_THREAD_STACK_SIZE_NONBLOCKING,
        Record           = FMOD_THREAD_STACK_SIZE_RECORD,
        Geometry         = FMOD_THREAD_STACK_SIZE_GEOMETRY,
        Profiler         = FMOD_THREAD_STACK_SIZE_PROFILER,
        StudioUpdate     = FMOD_THREAD_STACK_SIZE_STUDIO_UPDATE,
        StudioLoadBank   = FMOD_THREAD_STACK_SIZE_STUDIO_LOAD_BANK,
        StudioLoadSample = FMOD_THREAD_STACK_SIZE_STUDIO_LOAD_SAMPLE,
        Convolution1     = FMOD_THREAD_STACK_SIZE_CONVOLUTION1,
        Convolution2     = FMOD_THREAD_STACK_SIZE_CONVOLUTION2,
    }

    pub enum ThreadType: i32 {
        Mixer            = FMOD_THREAD_TYPE_MIXER,
        Feeder           = FMOD_THREAD_TYPE_FEEDER,
        Stream           = FMOD_THREAD_TYPE_STREAM,
        File             = FMOD_THREAD_TYPE_FILE,
        NonBlocking      = FMOD_THREAD_TYPE_NONBLOCKING,
        Record           = FMOD_THREAD_TYPE_RECORD,
        Geometry         = FMOD_THREAD_TYPE_GEOMETRY,
        Profiler         = FMOD_THREAD_TYPE_PROFILER,
        StudioUpdate     = FMOD_THREAD_TYPE_STUDIO_UPDATE,
        StudioLoadBank   = FMOD_THREAD_TYPE_STUDIO_LOAD_BANK,
        StudioLoadSample = FMOD_THREAD_TYPE_STUDIO_LOAD_SAMPLE,
        Convolution1     = FMOD_THREAD_TYPE_CONVOLUTION1,
        Convolution2     = FMOD_THREAD_TYPE_CONVOLUTION2,
    }

    pub enum ChannelControlType: i32 {
        Channel      = FMOD_CHANNELCONTROL_CHANNEL,
        ChannelGroup = FMOD_CHANNELCONTROL_CHANNELGROUP,
    }

    pub enum OutputType: i32 {
        AutoDetect   = FMOD_OUTPUTTYPE_AUTODETECT,
        Unknown      = FMOD_OUTPUTTYPE_UNKNOWN,
        NoSound      = FMOD_OUTPUTTYPE_NOSOUND,
        WavWriter    = FMOD_OUTPUTTYPE_WAVWRITER,
        NoSoundNrt   = FMOD_OUTPUTTYPE_NOSOUND_NRT,
        WavWriterNrt = FMOD_OUTPUTTYPE_WAVWRITER_NRT,
        Wasapi       = FMOD_OUTPUTTYPE_WASAPI,
        Asio         = FMOD_OUTPUTTYPE_ASIO,
        PulseAudio   = FMOD_OUTPUTTYPE_PULSEAUDIO,
        Alsa         = FMOD_OUTPUTTYPE_ALSA,
        CoreAudio    = FMOD_OUTPUTTYPE_COREAUDIO,
        AudioTrack   = FMOD_OUTPUTTYPE_AUDIOTRACK,
        OpenSl       = FMOD_OUTPUTTYPE_OPENSL,
        AudioOut     = FMOD_OUTPUTTYPE_AUDIOOUT,
        Audio3d      = FMOD_OUTPUTTYPE_AUDIO3D,
        WebAudio     = FMOD_OUTPUTTYPE_WEBAUDIO,
        NnAudio      = FMOD_OUTPUTTYPE_NNAUDIO,
        Winsonic     = FMOD_OUTPUTTYPE_WINSONIC,
        AAudio       = FMOD_OUTPUTTYPE_AAUDIO,
        AudioWorklet = FMOD_OUTPUTTYPE_AUDIOWORKLET,
    }

    pub enum DebugMode: i32 {
        Tty      = FMOD_DEBUG_MODE_TTY,
        File     = FMOD_DEBUG_MODE_FILE,
        Callback = FMOD_DEBUG_MODE_CALLBACK,
    }

    pub enum SpeakerMode: i32 {
        Default     = FMOD_SPEAKERMODE_DEFAULT,
        Raw         = FMOD_SPEAKERMODE_RAW,
        Mono        = FMOD_SPEAKERMODE_MONO,
        Stereo      = FMOD_SPEAKERMODE_STEREO,
        Quad        = FMOD_SPEAKERMODE_QUAD,
        Surround    = FMOD_SPEAKERMODE_SURROUND,
        Surround51  = FMOD_SPEAKERMODE_5POINT1,
        Surround71  = FMOD_SPEAKERMODE_7POINT1,
        Surround714 = FMOD_SPEAKERMODE_7POINT1POINT4,
    }

    pub enum Speaker: i32 {
        None          = FMOD_SPEAKER_NONE,
        FrontLeft     = FMOD_SPEAKER_FRONT_LEFT,
        FrontRight    = FMOD_SPEAKER_FRONT_RIGHT,
        FrontCenter   = FMOD_SPEAKER_FRONT_CENTER,
        LowFrequency  = FMOD_SPEAKER_LOW_FREQUENCY,
        SurroundLeft  = FMOD_SPEAKER_SURROUND_LEFT,
        SurroundRight = FMOD_SPEAKER_SURROUND_RIGHT,
        BackLeft      = FMOD_SPEAKER_BACK_LEFT,
        BackRight     = FMOD_SPEAKER_BACK_RIGHT,
        TopFrontLeft  = FMOD_SPEAKER_TOP_FRONT_LEFT,
        TopFrontRight = FMOD_SPEAKER_TOP_FRONT_RIGHT,
        TopBackLeft   = FMOD_SPEAKER_TOP_BACK_LEFT,
        TopBackRight  = FMOD_SPEAKER_TOP_BACK_RIGHT,
    }

    pub enum ChannelOrder: i32 {
        Default    = FMOD_CHANNELORDER_DEFAULT,
        WaveFormat = FMOD_CHANNELORDER_WAVEFORMAT,
        ProTools   = FMOD_CHANNELORDER_PROTOOLS,
        AllMono    = FMOD_CHANNELORDER_ALLMONO,
        AllStereo  = FMOD_CHANNELORDER_ALLSTEREO,
        Alsa       = FMOD_CHANNELORDER_ALSA,
    }

    pub enum PluginType: i32 {
        Output = FMOD_PLUGINTYPE_OUTPUT,
        Codec  = FMOD_PLUGINTYPE_CODEC,
        Dsp    = FMOD_PLUGINTYPE_DSP,
    }

    pub enum SoundType: i32 {
        Unknown         = FMOD_SOUND_TYPE_UNKNOWN,
        Aiff            = FMOD_SOUND_TYPE_AIFF,
        Asf             = FMOD_SOUND_TYPE_ASF,
        Dls             = FMOD_SOUND_TYPE_DLS,
        Flac            = FMOD_SOUND_TYPE_FLAC,
        Fsb             = FMOD_SOUND_TYPE_FSB,
        It              = FMOD_SOUND_TYPE_IT,
        Midi            = FMOD_SOUND_TYPE_MIDI,
        Mod             = FMOD_SOUND_TYPE_MOD,
        Mpeg            = FMOD_SOUND_TYPE_MPEG,
        OggVorbis       = FMOD_SOUND_TYPE_OGGVORBIS,
        Playlist        = FMOD_SOUND_TYPE_PLAYLIST,
        Raw             = FMOD_SOUND_TYPE_RAW,
        S3m             = FMOD_SOUND_TYPE_S3M,
        User            = FMOD_SOUND_TYPE_USER,
        Wav             = FMOD_SOUND_TYPE_WAV,
        Xm              = FMOD_SOUND_TYPE_XM,
        Xma             = FMOD_SOUND_TYPE_XMA,
        AudioQueue      = FMOD_SOUND_TYPE_AUDIOQUEUE,
        At9             = FMOD_SOUND_TYPE_AT9,
        Vorbis          = FMOD_SOUND_TYPE_VORBIS,
        MediaFoundation = FMOD_SOUND_TYPE_MEDIA_FOUNDATION,
        MediaCodec      = FMOD_SOUND_TYPE_MEDIACODEC,
        Fadpcm          = FMOD_SOUND_TYPE_FADPCM,
        Opus            = FMOD_SOUND_TYPE_OPUS,
    }

    pub enum SoundFormat: i32 {
        None      = FMOD_SOUND_FORMAT_NONE,
        Pcm8      = FMOD_SOUND_FORMAT_PCM8,
        Pcm16     = FMOD_SOUND_FORMAT_PCM16,
        Pcm24     = FMOD_SOUND_FORMAT_PCM24,
        Pcm32     = FMOD_SOUND_FORMAT_PCM32,
        PcmFloat  = FMOD_SOUND_FORMAT_PCMFLOAT,
        Bitstream = FMOD_SOUND_FORMAT_BITSTREAM,
    }

    pub enum OpenState: i32 {
        Ready       = FMOD_OPENSTATE_READY,
        Loading     = FMOD_OPENSTATE_LOADING,
        Error       = FMOD_OPENSTATE_ERROR,
        Connecting  = FMOD_OPENSTATE_CONNECTING,
        Buffering   = FMOD_OPENSTATE_BUFFERING,
        Seeking     = FMOD_OPENSTATE_SEEKING,
        Playing     = FMOD_OPENSTATE_PLAYING,
        SetPosition = FMOD_OPENSTATE_SETPOSITION,
    }

    pub enum SoundGroupBehavior: i32 {
        Fail        = FMOD_SOUNDGROUP_BEHAVIOR_FAIL,
        Mute        = FMOD_SOUNDGROUP_BEHAVIOR_MUTE,
        StealLowest = FMOD_SOUNDGROUP_BEHAVIOR_STEALLOWEST,
    }

    pub enum ChannelControlCallbackType: i32 {
        End          = FMOD_CHANNELCONTROL_CALLBACK_END,
        VirtualVoice = FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE,
        SyncPoint    = FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT,
        Occlusion    = FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION,
    }

    pub enum ChannelControlDspIndex: i32 {
        Head  = FMOD_CHANNELCONTROL_DSP_HEAD,
        Fader = FMOD_CHANNELCONTROL_DSP_FADER,
        Tail  = FMOD_CHANNELCONTROL_DSP_TAIL,
    }

    pub enum ErrorCallbackInstaceType: i32 {
        None                    = FMOD_ERRORCALLBACK_INSTANCETYPE_NONE,
        System                  = FMOD_ERRORCALLBACK_INSTANCETYPE_SYSTEM,
        Channel                 = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNEL,
        ChannelGroup            = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNELGROUP,
        ChannelControl          = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNELCONTROL,
        Sound                   = FMOD_ERRORCALLBACK_INSTANCETYPE_SOUND,
        SoundGroup              = FMOD_ERRORCALLBACK_INSTANCETYPE_SOUNDGROUP,
        Dsp                     = FMOD_ERRORCALLBACK_INSTANCETYPE_DSP,
        DspConnection           = FMOD_ERRORCALLBACK_INSTANCETYPE_DSPCONNECTION,
        Geometry                = FMOD_ERRORCALLBACK_INSTANCETYPE_GEOMETRY,
        Reverb3d                = FMOD_ERRORCALLBACK_INSTANCETYPE_REVERB3D,
        StudioSystem            = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_SYSTEM,
        StudioEventDescription  = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_EVENTDESCRIPTION,
        StudioEventInstance     = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_EVENTINSTANCE,
        StudioParameterInstance = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_PARAMETERINSTANCE,
        StudioBus               = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_BUS,
        StudioVca               = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_VCA,
        StudioBank              = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_BANK,
        StudioCommandReplay     = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_COMMANDREPLAY,
    }

    pub enum DspResampler: i32 {
        Default  = FMOD_DSP_RESAMPLER_DEFAULT,
        NoInterp = FMOD_DSP_RESAMPLER_NOINTERP,
        Linear   = FMOD_DSP_RESAMPLER_LINEAR,
        Cubic    = FMOD_DSP_RESAMPLER_CUBIC,
        Spline   = FMOD_DSP_RESAMPLER_SPLINE,
    }

    pub enum DspConnectionType: i32 {
        Standard      = FMOD_DSPCONNECTION_TYPE_STANDARD,
        Sidechain     = FMOD_DSPCONNECTION_TYPE_SIDECHAIN,
        Send          = FMOD_DSPCONNECTION_TYPE_SEND,
        SendSidechain = FMOD_DSPCONNECTION_TYPE_SEND_SIDECHAIN,
    }

    pub enum TagType: i32 {
        Unknown       = FMOD_TAGTYPE_UNKNOWN,
        Id3v1         = FMOD_TAGTYPE_ID3V1,
        Id3v2         = FMOD_TAGTYPE_ID3V2,
        VorbisComment = FMOD_TAGTYPE_VORBISCOMMENT,
        Shoutcast     = FMOD_TAGTYPE_SHOUTCAST,
        Icecast       = FMOD_TAGTYPE_ICECAST,
        Asf           = FMOD_TAGTYPE_ASF,
        Midi          = FMOD_TAGTYPE_MIDI,
        Playlist      = FMOD_TAGTYPE_PLAYLIST,
        Fmod          = FMOD_TAGTYPE_FMOD,
        User          = FMOD_TAGTYPE_USER,
    }

    pub enum TagDataType: i32 {
        Binary        = FMOD_TAGDATATYPE_BINARY,
        Int           = FMOD_TAGDATATYPE_INT,
        Float         = FMOD_TAGDATATYPE_FLOAT,
        String        = FMOD_TAGDATATYPE_STRING,
        StringUtf16   = FMOD_TAGDATATYPE_STRING_UTF16,
        StringUtf16be = FMOD_TAGDATATYPE_STRING_UTF16BE,
        StringUtf8    = FMOD_TAGDATATYPE_STRING_UTF8,
    }

    pub enum PortType: i32 {
        Music          = FMOD_PORT_TYPE_MUSIC,
        CopyrightMusic = FMOD_PORT_TYPE_COPYRIGHT_MUSIC,
        Voice          = FMOD_PORT_TYPE_VOICE,
        Controller     = FMOD_PORT_TYPE_CONTROLLER,
        Personal       = FMOD_PORT_TYPE_PERSONAL,
        Vibration      = FMOD_PORT_TYPE_VIBRATION,
        Aux            = FMOD_PORT_TYPE_AUX,
    }
}
