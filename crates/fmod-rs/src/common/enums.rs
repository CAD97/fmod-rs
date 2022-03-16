use crate::raw::*;

macro_rules! enum_struct {
    {$(
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: $Repr:ty {$(
            $(#[$vmeta:meta])*
            $Variant:ident = $value:expr,
        )*}
    )*} => {$(
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        $vis struct $Name {
            raw: $Repr,
        }

        impl $Name {
            raw! {
                #[allow(clippy::missing_safety_doc)]
                pub const unsafe fn from_raw(raw: $Repr) -> Self {
                    Self { raw }
                }
            }

            raw! {
                pub const fn into_raw(this: Self) -> $Repr {
                    this.raw
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

    // FMOD_CHANNELORDER
    // FMOD_PLUGINTYPE
    // FMOD_SOUND_TYPE
    // FMOD_SOUND_FORMAT
    // FMOD_OPENSTATE
    // FMOD_SOUNDGROUP_BEHAVIOR
    // FMOD_CHANNELCONTROL_CALLBACK_TYPE
    // FMOD_CHANNELCONTROL_DSP_INDEX
    // FMOD_ERRORCALLBACK_INSTANCETYPE
    // FMOD_DSP_RESAMPLER
    // FMOD_DSPCONNECTION_TYPE
    // FMOD_TAGTYPE
    // FMOD_PORT_TYPE
}
