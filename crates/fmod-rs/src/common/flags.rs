use {
    fmod::raw::*,
    std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

macro_rules! ops {
    ($Name:ty: $Op:ident $fn_op:ident $op:tt) => {
        impl $Op for $Name {
            type Output = $Name;
            fn $fn_op(self) -> $Name {
                let raw = $op <$Name>::into_raw(self);
                unsafe { <$Name>::from_raw(raw) }
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
                unsafe { <$Name>::from_raw(raw) }
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
        $vis:vis struct $Name:ident: $Repr:ty {$(
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

                pub const fn into_raw(this: Self) -> $Repr {
                    this.raw
                }
            }

            pub fn is_set(self, variant: Self) -> bool {
                self & variant == variant
            }

            $(
                $(#[$vmeta])*
                #[allow(non_upper_case_globals)]
                pub const $Variant: Self = unsafe { Self::from_raw($value) };
            )*
        }

        ops!($Name: BitAnd bitand & BitAndAssign bitand_assign);
        ops!($Name: BitOr bitor | BitOrAssign bitor_assign);
        ops!($Name: BitXor bitxor ^ BitXorAssign bitxor_assign);
        ops!($Name: Not not !);
    )*};
}

flags! {
    pub struct DebugFlags: u32 {
        LevelNone          = FMOD_DEBUG_LEVEL_NONE,
        LevelError         = FMOD_DEBUG_LEVEL_ERROR,
        LevelWarning       = FMOD_DEBUG_LEVEL_WARNING,
        LevelLog           = FMOD_DEBUG_LEVEL_LOG,
        TypeMemory         = FMOD_DEBUG_TYPE_MEMORY,
        TypeFile           = FMOD_DEBUG_TYPE_FILE,
        TypeCodec          = FMOD_DEBUG_TYPE_CODEC,
        TypeTrace          = FMOD_DEBUG_TYPE_TRACE,
        DisplayTimestamps  = FMOD_DEBUG_DISPLAY_TIMESTAMPS,
        DisplayLinenumbers = FMOD_DEBUG_DISPLAY_LINENUMBERS,
        DisplayThread      = FMOD_DEBUG_DISPLAY_THREAD,
    }

    pub struct MemoryType: u32 {
        Normal       = FMOD_MEMORY_NORMAL,
        StreamFile   = FMOD_MEMORY_STREAM_FILE,
        StreamDecode = FMOD_MEMORY_STREAM_DECODE,
        SampleData   = FMOD_MEMORY_SAMPLEDATA,
        DspBuffer    = FMOD_MEMORY_DSP_BUFFER,
        Plugin       = FMOD_MEMORY_PLUGIN,
        Persistent   = FMOD_MEMORY_PERSISTENT,
        All          = FMOD_MEMORY_ALL,
    }

    pub struct InitFlags: u32 {
        Normal                 = FMOD_INIT_NORMAL,
        StreamFromUpdate       = FMOD_INIT_STREAM_FROM_UPDATE,
        MixFromUpdate          = FMOD_INIT_MIX_FROM_UPDATE,
        RightHanded3d          = FMOD_INIT_3D_RIGHTHANDED,
        ChannelLowpass         = FMOD_INIT_CHANNEL_LOWPASS,
        ChannelDistanceFilter  = FMOD_INIT_CHANNEL_DISTANCEFILTER,
        ProfileEnable          = FMOD_INIT_PROFILE_ENABLE,
        Vol0BecomesVirtual     = FMOD_INIT_VOL0_BECOMES_VIRTUAL,
        GeometryUseclosest     = FMOD_INIT_GEOMETRY_USECLOSEST,
        PreferDolbyDownmix     = FMOD_INIT_PREFER_DOLBY_DOWNMIX,
        ThreadUnsafe           = FMOD_INIT_THREAD_UNSAFE,
        ProfileMeterAll        = FMOD_INIT_PROFILE_METER_ALL,
        MemoryTracking         = FMOD_INIT_MEMORY_TRACKING,
    }


    pub struct SystemCallbackType: u32 {
        DeviceListChanged      = FMOD_SYSTEM_CALLBACK_DEVICELISTCHANGED,
        DeviceLost             = FMOD_SYSTEM_CALLBACK_DEVICELOST,
        MemoryAllocationFailed = FMOD_SYSTEM_CALLBACK_MEMORYALLOCATIONFAILED,
        ThreadCreated          = FMOD_SYSTEM_CALLBACK_THREADCREATED,
        BadDspConnection       = FMOD_SYSTEM_CALLBACK_BADDSPCONNECTION,
        PreMix                 = FMOD_SYSTEM_CALLBACK_PREMIX,
        PostMix                = FMOD_SYSTEM_CALLBACK_POSTMIX,
        Error                  = FMOD_SYSTEM_CALLBACK_ERROR,
        MidMix                 = FMOD_SYSTEM_CALLBACK_MIDMIX,
        ThreadDestroyed        = FMOD_SYSTEM_CALLBACK_THREADDESTROYED,
        PreUpdate              = FMOD_SYSTEM_CALLBACK_PREUPDATE,
        PostUpdate             = FMOD_SYSTEM_CALLBACK_POSTUPDATE,
        RecordListChanged      = FMOD_SYSTEM_CALLBACK_RECORDLISTCHANGED,
        BufferedNoMix          = FMOD_SYSTEM_CALLBACK_BUFFEREDNOMIX,
        DeviceReinitialize     = FMOD_SYSTEM_CALLBACK_DEVICEREINITIALIZE,
        OutputUnderrun         = FMOD_SYSTEM_CALLBACK_OUTPUTUNDERRUN,
        All                    = FMOD_SYSTEM_CALLBACK_ALL,
    }


    pub struct ChannelMask: u32 {
        FrontLeft       = FMOD_CHANNELMASK_FRONT_LEFT,
        FrontRight      = FMOD_CHANNELMASK_FRONT_RIGHT,
        FrontCenter     = FMOD_CHANNELMASK_FRONT_CENTER,
        LowFrequency    = FMOD_CHANNELMASK_LOW_FREQUENCY,
        SurroundLeft    = FMOD_CHANNELMASK_SURROUND_LEFT,
        SurroundRight   = FMOD_CHANNELMASK_SURROUND_RIGHT,
        BackLeft        = FMOD_CHANNELMASK_BACK_LEFT,
        BackRight       = FMOD_CHANNELMASK_BACK_RIGHT,
        BackCenter      = FMOD_CHANNELMASK_BACK_CENTER,
        Mono            = FMOD_CHANNELMASK_MONO,
        Stereo          = FMOD_CHANNELMASK_STEREO,
        Lrc             = FMOD_CHANNELMASK_LRC,
        Quad            = FMOD_CHANNELMASK_QUAD,
        Surround        = FMOD_CHANNELMASK_SURROUND,
        Surround51      = FMOD_CHANNELMASK_5POINT1,
        Surround51Rears = FMOD_CHANNELMASK_5POINT1_REARS,
        Surround70      = FMOD_CHANNELMASK_7POINT0,
        Surround71      = FMOD_CHANNELMASK_7POINT1,
    }


    pub struct ThreadAffinity: u64 {
        // Platform agnostic thread groupings
        GroupDefault = FMOD_THREAD_AFFINITY_GROUP_DEFAULT,
        GroupA       = FMOD_THREAD_AFFINITY_GROUP_A,
        GroupB       = FMOD_THREAD_AFFINITY_GROUP_B,
        GroupC       = FMOD_THREAD_AFFINITY_GROUP_C,
        // Thread defaults
        Mixer            = FMOD_THREAD_AFFINITY_MIXER,
        Feeder           = FMOD_THREAD_AFFINITY_FEEDER,
        Stream           = FMOD_THREAD_AFFINITY_STREAM,
        File             = FMOD_THREAD_AFFINITY_FILE,
        Nonblocking      = FMOD_THREAD_AFFINITY_NONBLOCKING,
        Record           = FMOD_THREAD_AFFINITY_RECORD,
        Geometry         = FMOD_THREAD_AFFINITY_GEOMETRY,
        Profiler         = FMOD_THREAD_AFFINITY_PROFILER,
        StudioUpdate     = FMOD_THREAD_AFFINITY_STUDIO_UPDATE,
        StudioLoadBank   = FMOD_THREAD_AFFINITY_STUDIO_LOAD_BANK,
        StudioLoadSample = FMOD_THREAD_AFFINITY_STUDIO_LOAD_SAMPLE,
        Convolution1     = FMOD_THREAD_AFFINITY_CONVOLUTION1,
        Convolution2     = FMOD_THREAD_AFFINITY_CONVOLUTION2,
        // Core mask, valid up to 1 << 62
        CoreAll = 0,
        Core0   = 1 << 0,
        Core1   = 1 << 1,
        Core2   = 1 << 2,
        Core3   = 1 << 3,
        Core4   = 1 << 4,
        Core5   = 1 << 5,
        Core6   = 1 << 6,
        Core7   = 1 << 7,
        Core8   = 1 << 8,
        Core9   = 1 << 9,
        Core10  = 1 << 10,
        Core11  = 1 << 11,
        Core12  = 1 << 12,
        Core13  = 1 << 13,
        Core14  = 1 << 14,
        Core15  = 1 << 15,
    }
}

impl ThreadAffinity {
    #[allow(non_snake_case)]
    pub const fn Core(n: u8) -> ThreadAffinity {
        if n <= 62 {
            unsafe { ThreadAffinity::from_raw(1 << n) }
        } else {
            panic!("thread affinity to core >62 given (nice CPU btw)")
        }
    }
}
