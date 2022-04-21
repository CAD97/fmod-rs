#[cfg(doc)]
use fmod::{Mode, OutputType, SpeakerMode, System, ThreadType};
use {
    fmod::raw::*,
    std::{
        fmt,
        ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
    },
};

macro_rules! ops {
    ($Name:ty: $Op:ident $fn_op:ident $op:tt) => {
        impl $Op for $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                let raw = $op <$Name>::into_raw(self);
                <$Name>::from_raw(raw)
            }
        }

        impl $Op for &'_ $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                $op *self
            }
        }
    };
    ($Name:ty: $Op:ident $fn_op:ident $op:tt $OpAssign:ident $fn_op_assign:ident) => {
        impl $Op for $Name {
            type Output = $Name;
            fn $fn_op(self, rhs: $Name) -> $Name {
                let raw = <$Name>::into_raw(self) $op <$Name>::into_raw(rhs);
                <$Name>::from_raw(raw)
            }
        }

        impl $Op<&$Name> for $Name {
            type Output = $Name;
            fn $fn_op(self, rhs: &$Name) -> $Name {
                self $op *rhs
            }
        }

        impl $Op<$Name> for &$Name {
            type Output = $Name;
            fn $fn_op(self, rhs: $Name) -> $Name {
                *self $op rhs
            }
        }

        impl $Op<&$Name> for &$Name {
            type Output = $Name;
            fn $fn_op(self, rhs: &$Name) -> $Name {
                *self $op *rhs
            }
        }

        impl $OpAssign for $Name {
            fn $fn_op_assign(&mut self, rhs: $Name) {
                *self = *self $op rhs;
            }
        }

        impl $OpAssign<&$Name> for $Name {
            fn $fn_op_assign(&mut self, rhs: &$Name) {
                *self = *self $op *rhs;
            }
        }
    };
}

macro_rules! flags {
    {$(
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident: $Raw:ty {$(
            $(#[$($vmeta:tt)*])*
            $Variant:ident = $value:expr,
        )*}
    )*} => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $Name {
            raw: $Raw,
        }

        impl $Name {
            $(
                flags! {@stripdefault
                    $(#[$($vmeta)*])*
                    #[allow(non_upper_case_globals)]
                    pub const $Variant: Self = Self::from_raw($value);
                }
            )*
        }

        impl $Name {
            raw! {
                pub const fn zeroed() -> $Name {
                    Self::from_raw(0)
                }
            }
            raw! {
                pub const fn from_raw(raw: $Raw) -> $Name {
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn from_raw_ref(raw: &$Raw) -> &$Name {
                    unsafe { &*(raw as *const $Raw as *const $Name ) }
                }
            }
            raw! {
                pub fn from_raw_mut(raw: &mut $Raw) -> &mut $Name {
                    unsafe { &mut *(raw as *mut $Raw as *mut $Name ) }
                }
            }
            raw! {
                pub const fn into_raw(self) -> $Raw {
                    unsafe { ::std::mem::transmute(self) }
                }
            }
            raw! {
                pub const fn as_raw(&self) -> &$Raw {
                    unsafe { &*(self as *const $Name as *const $Raw ) }
                }
            }
            raw! {
                pub fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name as *mut $Raw ) }
                }
            }

            pub fn is_set(self, variant: Self) -> bool {
                self & variant == variant
            }
        }

        ops!($Name: BitAnd bitand & BitAndAssign bitand_assign);
        ops!($Name: BitOr bitor | BitOrAssign bitor_assign);
        ops!($Name: BitXor bitxor ^ BitXorAssign bitxor_assign);
        ops!($Name: Not not !);

        impl fmt::Debug for $Name {
            #[allow(unreachable_patterns)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self {
                    $($Name::$Variant => f.debug_struct(stringify!($Variant)).finish(),)*
                    _ => f.debug_struct(stringify!($Name)).field("raw", &self.raw).finish(),
                }
            }
        }

        flags! {@default $Name {$(
            $(#[$($vmeta)*])*
            $Variant = $value,
        )*}}
    )*};

    {@default $Name:ident {}} => {};

    {@default $Name:ident {
        #[default]
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        #[doc = concat!("[`", stringify!($Name), "::", stringify!($Variant), "`]")]
        impl Default for $Name {
            fn default() -> $Name {
                $Name::$Variant
            }
        }
        flags! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@default $Name:ident {
        $(#[$meta:meta])*
        $Variant:ident = $value:expr,
        $(
            $(#[$($vmeta:tt)*])*
            $VVariant:ident = $vvalue:expr,
        )*
    }} => {
        flags! { @default $Name { $($(#[$($vmeta)*])* $VVariant = $vvalue,)* } }
    };

    {@stripdefault #[default] $($tt:tt)*} => { $($tt)* };
    {@stripdefault $($tt:tt)*} => { $($tt)* };
}

flags! {
    /// Specify the requested information to be output when using the logging version of FMOD.
    pub struct DebugFlags: u32 {
        /// Disable all messages.
        LevelNone          = FMOD_DEBUG_LEVEL_NONE,
        /// Enable only error messages.
        LevelError         = FMOD_DEBUG_LEVEL_ERROR,
        /// Enable warning and error messages.
        LevelWarning       = FMOD_DEBUG_LEVEL_WARNING,
        #[default]
        /// Enable informational, warning and error messages (default).
        LevelLog           = FMOD_DEBUG_LEVEL_LOG,
        /// Verbose logging for memory operations, only use this if you are debugging a memory related issue.
        TypeMemory         = FMOD_DEBUG_TYPE_MEMORY,
        /// Verbose logging for file access, only use this if you are debugging a file related issue.
        TypeFile           = FMOD_DEBUG_TYPE_FILE,
        /// Verbose logging for codec initialization, only use this if you are debugging a codec related issue.
        TypeCodec          = FMOD_DEBUG_TYPE_CODEC,
        /// Verbose logging for internal errors, use this for tracking the origin of error codes.
        TypeTrace          = FMOD_DEBUG_TYPE_TRACE,
        /// Display the time stamp of the log message in milliseconds.
        DisplayTimestamps  = FMOD_DEBUG_DISPLAY_TIMESTAMPS,
        /// Display the source code file and line number for where the message originated.
        DisplayLinenumbers = FMOD_DEBUG_DISPLAY_LINENUMBERS,
        /// Display the thread ID of the calling function that generated the message.
        DisplayThread      = FMOD_DEBUG_DISPLAY_THREAD,
    }

    /// Bitfields for memory allocation type being passed into FMOD memory callbacks.
    pub struct MemoryType: u32 {
        #[default]
        /// Standard memory.
        Normal       = FMOD_MEMORY_NORMAL,
        /// Stream file buffer, size controllable with [System::set_stream_buffer_size].
        StreamFile   = FMOD_MEMORY_STREAM_FILE,
        /// Stream decode buffer, size controllable with [CreateSoundExInfo::decode_buffer_size].
        StreamDecode = FMOD_MEMORY_STREAM_DECODE,
        /// Sample data buffer. Raw audio data, usually PCM/MPEG/ADPCM/XMA data.
        SampleData   = FMOD_MEMORY_SAMPLEDATA,
        /// Deprecated.
        DspBuffer    = FMOD_MEMORY_DSP_BUFFER,
        /// Memory allocated by a third party plugin.
        Plugin       = FMOD_MEMORY_PLUGIN,
        /// Persistent memory. Memory will be freed when the system is freed.
        Persistent   = FMOD_MEMORY_PERSISTENT,
        /// Mask specifying all memory types.
        All          = FMOD_MEMORY_ALL,
    }

    /// Configuration flags used when initializing the System object.
    pub struct InitFlags: u32 {
        #[default]
        /// Initialize normally
        Normal                 = FMOD_INIT_NORMAL,
        /// No stream thread is created internally. Streams are driven from [System::update]. Mainly used with non-realtime outputs.
        StreamFromUpdate       = FMOD_INIT_STREAM_FROM_UPDATE,
        /// No mixer thread is created internally. Mixing is driven from [System::update]. Only applies to polling based output modes such as [OutputType::NoSound], [OutputType::WavWriter].
        MixFromUpdate          = FMOD_INIT_MIX_FROM_UPDATE,
        /// 3D calculations will be performed in right-handed coordinates.
        RightHanded3d          = FMOD_INIT_3D_RIGHTHANDED,
        /// Enables usage of [Channel::set_low_pass_gain], [Channel::set_3d_occlusion], or automatic usage by the [Geometry] API. All voices will add a software lowpass filter effect into the DSP chain which is idle unless one of the previous functions/features are used.
        ChannelLowpass         = FMOD_INIT_CHANNEL_LOWPASS,
        /// All [Mode::D3] based voices will add a software lowpass and highpass filter effect into the DSP chain which will act as a distance-automated bandpass filter. Use [System::set_advanced_settings] to adjust the center frequency.
        ChannelDistanceFilter  = FMOD_INIT_CHANNEL_DISTANCEFILTER,
        /// Enable TCP/IP based host which allows FMOD Studio or FMOD Profiler to connect to it, and view memory, CPU and the DSP network graph in real-time.
        ProfileEnable          = FMOD_INIT_PROFILE_ENABLE,
        /// Any sounds that are 0 volume will go virtual and not be processed except for having their positions updated virtually. Use [System::set_advanced_settings] to adjust what volume besides zero to switch to virtual at.
        Vol0BecomesVirtual     = FMOD_INIT_VOL0_BECOMES_VIRTUAL,
        /// With the geometry engine, only process the closest polygon rather than accumulating all polygons the sound to listener line intersects.
        GeometryUseClosest     = FMOD_INIT_GEOMETRY_USECLOSEST,
        /// When using [SpeakerMode::Surround51] with a stereo output device, use the Dolby Pro Logic II downmix algorithm instead of the default stereo downmix algorithm.
        PreferDolbyDownmix     = FMOD_INIT_PREFER_DOLBY_DOWNMIX,
        /// Disables thread safety for API calls. Only use this if FMOD is being called from a single thread, and if Studio API is not being used!
        ThreadUnsafe           = FMOD_INIT_THREAD_UNSAFE,
        /// Slower, but adds level metering for every single DSP unit in the graph. Use [DSP::set_metering_enabled] to turn meters off individually. Setting this flag implies [InitFlags::ProfileEnable].
        ProfileMeterAll        = FMOD_INIT_PROFILE_METER_ALL,
        /// Enables memory allocation tracking. Currently this is only useful when using the Studio API. Increases memory footprint and reduces performance. This flag is implied by [studio::InitFlags::MemoryTracking].
        MemoryTracking         = FMOD_INIT_MEMORY_TRACKING,
    }

    /// Types of callbacks called by the System.
    ///
    /// Using [SystemCallbackType::All] or [SystemCallbackType::DeviceListChanged] will disable any automated device ejection/insertion handling. Use this callback to control the behavior yourself.
    /// Using [SystemCallbackType::DeviceListChanged] (Mac only) requires the application to be running an event loop which will allow external changes to device list to be detected.
    pub struct SystemCallbackType: u32 {
        /// Called from [System::update] when the enumerated list of devices has changed. Called from the main (calling) thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        DeviceListChanged      = FMOD_SYSTEM_CALLBACK_DEVICELISTCHANGED,
        /// Deprecated.
        DeviceLost             = FMOD_SYSTEM_CALLBACK_DEVICELOST,
        /// Called directly when a memory allocation fails.
        MemoryAllocationFailed = FMOD_SYSTEM_CALLBACK_MEMORYALLOCATIONFAILED,
        /// Called from the game thread when a thread is created.
        ThreadCreated          = FMOD_SYSTEM_CALLBACK_THREADCREATED,
        /// Deprecated.
        BadDspConnection       = FMOD_SYSTEM_CALLBACK_BADDSPCONNECTION,
        /// Called from the mixer thread before it starts the next block.
        PreMix                 = FMOD_SYSTEM_CALLBACK_PREMIX,
        /// Called from the mixer thread after it finishes a block.
        PostMix                = FMOD_SYSTEM_CALLBACK_POSTMIX,
        /// Called directly when an API function returns an error, including delayed async functions.
        Error                  = FMOD_SYSTEM_CALLBACK_ERROR,
        /// Called from the mixer thread after clocks have been updated before the main mix occurs.
        MidMix                 = FMOD_SYSTEM_CALLBACK_MIDMIX,
        /// Called from the game thread when a thread is destroyed.
        ThreadDestroyed        = FMOD_SYSTEM_CALLBACK_THREADDESTROYED,
        /// Called at start of [System::update] from the main (calling) thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        PreUpdate              = FMOD_SYSTEM_CALLBACK_PREUPDATE,
        /// Called at end of [System::update] from the main (calling) thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        PostUpdate             = FMOD_SYSTEM_CALLBACK_POSTUPDATE,
        /// Called from [System::update] when the enumerated list of recording devices has changed. Called from the main (calling) thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        RecordListChanged      = FMOD_SYSTEM_CALLBACK_RECORDLISTCHANGED,
        /// Called from the feeder thread after audio was consumed from the ring buffer, but not enough to allow another mix to run.
        BufferedNoMix          = FMOD_SYSTEM_CALLBACK_BUFFEREDNOMIX,
        /// Called from [System::update] when an output device is re-initialized. Called from the main (calling) thread when set from the Core API or Studio API in synchronous mode, and from the Studio Update Thread when in default / async mode.
        DeviceReinitialize     = FMOD_SYSTEM_CALLBACK_DEVICEREINITIALIZE,
        /// Called from the mixer thread when the device output attempts to read more samples than are available in the output buffer.
        OutputUnderrun         = FMOD_SYSTEM_CALLBACK_OUTPUTUNDERRUN,
        /// Mask representing all callback types.
        All                    = FMOD_SYSTEM_CALLBACK_ALL,
    }

    /// Flags that describe the speakers present in a given signal.
    pub struct ChannelMask: u32 {
        /// Front left channel.
        FrontLeft       = FMOD_CHANNELMASK_FRONT_LEFT,
        /// Front right channel.
        FrontRight      = FMOD_CHANNELMASK_FRONT_RIGHT,
        /// Front center channel.
        FrontCenter     = FMOD_CHANNELMASK_FRONT_CENTER,
        /// Low frequency channel.
        LowFrequency    = FMOD_CHANNELMASK_LOW_FREQUENCY,
        /// Surround left channel.
        SurroundLeft    = FMOD_CHANNELMASK_SURROUND_LEFT,
        /// Surround right channel.
        SurroundRight   = FMOD_CHANNELMASK_SURROUND_RIGHT,
        /// Back left channel.
        BackLeft        = FMOD_CHANNELMASK_BACK_LEFT,
        /// Back right channel.
        BackRight       = FMOD_CHANNELMASK_BACK_RIGHT,
        /// Back center channel, not represented in any [SpeakerMode].
        BackCenter      = FMOD_CHANNELMASK_BACK_CENTER,
        /// Mono channel mask.
        Mono            = FMOD_CHANNELMASK_MONO,
        /// Stereo channel mask.
        Stereo          = FMOD_CHANNELMASK_STEREO,
        /// Left / right / center channel mask.
        Lrc             = FMOD_CHANNELMASK_LRC,
        /// Quadphonic channel mask.
        Quad            = FMOD_CHANNELMASK_QUAD,
        /// 5.0 surround channel mask.
        Surround        = FMOD_CHANNELMASK_SURROUND,
        /// 5.1 surround channel mask.
        Surround51      = FMOD_CHANNELMASK_5POINT1,
        /// 5.1 surround channel mask, using rears instead of surrounds.
        Surround51Rears = FMOD_CHANNELMASK_5POINT1_REARS,
        /// 7.0 surround channel mask.
        Surround70      = FMOD_CHANNELMASK_7POINT0,
        /// 7.1 surround channel mask.
        Surround71      = FMOD_CHANNELMASK_7POINT1,
    }

    /// Bitfield for specifying the CPU core a given thread runs on.
    ///
    /// The platform agnostic thread groups, A, B and C give recommendations about FMOD threads that should be separated from one another.
    /// Platforms with fixed CPU core counts will try to honor this request, those that don't will leave affinity to the operating system.
    /// See the FMOD platform specific docs for each platform to see how the groups map to cores.
    ///
    /// If an explicit core affinity is given, i.e. [ThreadAffinity::Core11] and that core is unavailable a fatal error will be produced.
    ///
    /// Explicit core assignment up to [ThreadAffinity::Core(61)][Self::Core] is supported for platforms with that many cores.
    pub struct ThreadAffinity: u64 {
        // Platform agnostic thread groupings
        #[default]
        /// For a given thread use the default listed below, i.e. [ThreadType::Mixer] uses [ThreadAffinity::Mixer].
        GroupDefault = FMOD_THREAD_AFFINITY_GROUP_DEFAULT,
        /// Grouping A is recommended to isolate the mixer thread [ThreadType::Mixer].
        GroupA       = FMOD_THREAD_AFFINITY_GROUP_A,
        /// Grouping B is recommended to isolate the Studio update thread [ThreadType::StudioUpdate].
        GroupB       = FMOD_THREAD_AFFINITY_GROUP_B,
        /// Grouping C is recommended for all remaining threads.
        GroupC       = FMOD_THREAD_AFFINITY_GROUP_C,
        // Thread defaults
        /// Default affinity for [ThreadType::Mixer].
        Mixer            = FMOD_THREAD_AFFINITY_MIXER,
        /// Default affinity for [ThreadType::Feeder].
        Feeder           = FMOD_THREAD_AFFINITY_FEEDER,
        /// Default affinity for [ThreadType::Stream].
        Stream           = FMOD_THREAD_AFFINITY_STREAM,
        /// Default affinity for [ThreadType::File].
        File             = FMOD_THREAD_AFFINITY_FILE,
        /// Default affinity for [ThreadType::NonBlocking].
        NonBlocking      = FMOD_THREAD_AFFINITY_NONBLOCKING,
        /// Default affinity for [ThreadType::Record].
        Record           = FMOD_THREAD_AFFINITY_RECORD,
        /// Default affinity for [ThreadType::Geometry].
        Geometry         = FMOD_THREAD_AFFINITY_GEOMETRY,
        /// Default affinity for [ThreadType::Profiler].
        Profiler         = FMOD_THREAD_AFFINITY_PROFILER,
        /// Default affinity for [ThreadType::StudioUpdate].
        StudioUpdate     = FMOD_THREAD_AFFINITY_STUDIO_UPDATE,
        /// Default affinity for [ThreadType::StudioLoadBank].
        StudioLoadBank   = FMOD_THREAD_AFFINITY_STUDIO_LOAD_BANK,
        /// Default affinity for [ThreadType::StudioLoadSample].
        StudioLoadSample = FMOD_THREAD_AFFINITY_STUDIO_LOAD_SAMPLE,
        /// Default affinity for [ThreadType::Convolution1].
        Convolution1     = FMOD_THREAD_AFFINITY_CONVOLUTION1,
        /// Default affinity for [ThreadType::Convolution2].
        Convolution2     = FMOD_THREAD_AFFINITY_CONVOLUTION2,
        // Core mask, valid up to 1 << 62
        /// Assign to all cores.
        CoreAll = 0,
        /// Assign to core 0.
        Core0   = 1 << 0,
        /// Assign to core 1.
        Core1   = 1 << 1,
        /// Assign to core 2.
        Core2   = 1 << 2,
        /// Assign to core 3.
        Core3   = 1 << 3,
        /// Assign to core 4.
        Core4   = 1 << 4,
        /// Assign to core 5.
        Core5   = 1 << 5,
        /// Assign to core 6.
        Core6   = 1 << 6,
        /// Assign to core 7.
        Core7   = 1 << 7,
        /// Assign to core 8.
        Core8   = 1 << 8,
        /// Assign to core 9.
        Core9   = 1 << 9,
        /// Assign to core 10.
        Core10  = 1 << 10,
        /// Assign to core 11.
        Core11  = 1 << 11,
        /// Assign to core 12.
        Core12  = 1 << 12,
        /// Assign to core 13.
        Core13  = 1 << 13,
        /// Assign to core 14.
        Core14  = 1 << 14,
        /// Assign to core 15.
        Core15  = 1 << 15,
    }
}

impl ThreadAffinity {
    #[allow(non_snake_case)]
    pub const fn Core(n: u8) -> ThreadAffinity {
        if n <= 62 {
            ThreadAffinity::from_raw(1 << n)
        } else {
            panic!("thread affinity to core >62 given (nice CPU btw)")
        }
    }
}
