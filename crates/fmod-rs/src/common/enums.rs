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
    // FMOD_DRIVER_STATE

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

    // FMOD_THREAD_PRIORITY
    // FMOD_THREAD_STACK_SIZE

    // FMOD_THREAD_TYPE
    // FMOD_CHANNELCONTROL_TYPE
    // FMOD_OUTPUTTYPE
    // FMOD_SPEAKERMODE
    // FMOD_SPEAKER
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
