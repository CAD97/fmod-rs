//! Functionality relating to FMOD's thread usage.

use fmod::{raw::*, *};

/// Specify the affinity, priority and stack size for all FMOD created threads.
///
/// Affinity can be specified using one (or more) of the [ThreadAffinity]
/// constants or by providing the bits explicitly, i.e. (1<<3) for logical
/// core three (core affinity is zero based).  
/// See platform documentation for details on the available cores for a
/// given device.
///
/// Priority can be specified using one of the [ThreadPriority] constants or
/// by providing the value explicitly, i.e. (-2) for the lowest thread
/// priority on Windows.  
/// See platform documentation for details on the available priority values
/// for a given operating system.
///
/// Stack size can be specified explicitly, however for each thread you
/// should provide a size equal to or larger than the expected default or
/// risk causing a stack overflow at runtime.
///
/// # Safety
///
/// This function must be called before any FMOD [System] object is created.
pub unsafe fn set_attributes(
    kind: ThreadType,
    affinity: ThreadAffinity,
    priority: ThreadPriority,
    stack_size: ThreadStackSize,
) -> Result {
    ffi!(FMOD_Thread_SetAttributes(
        kind.into_raw(),
        affinity.into_raw(),
        priority.into_raw(),
        stack_size.into_raw()
    ))?;
    Ok(())
}

// -------------------------------------------------------------------------------------------------

fmod_enum! {
    /// Named constants for threads created at runtime.
    pub enum ThreadType: FMOD_THREAD_TYPE {
        /// Thread responsible for mixing and processing blocks of audio.
        Mixer            = FMOD_THREAD_TYPE_MIXER,
        /// Thread used by some output plugins for transferring buffered audio from [ThreadType::Mixer] to the sound output device.
        Feeder           = FMOD_THREAD_TYPE_FEEDER,
        /// Thread that decodes compressed audio to PCM for Sounds created as [Mode::CreateStream].
        Stream           = FMOD_THREAD_TYPE_STREAM,
        /// Thread that reads compressed audio from disk to be consumed by [ThreadType::Stream].
        File             = FMOD_THREAD_TYPE_FILE,
        /// Thread that processes the creation of Sounds asynchronously when opened with [Mode::NonBlocking].
        NonBlocking      = FMOD_THREAD_TYPE_NONBLOCKING,
        /// Thread used by some output plugins for transferring audio from a microphone to [ThreadType::Mixer].
        Record           = FMOD_THREAD_TYPE_RECORD,
        /// Thread used by the [Geometry] system for performing background calculations.
        Geometry         = FMOD_THREAD_TYPE_GEOMETRY,
        /// Thread for network communication when using [InitFlags::ProfileEnable].
        Profiler         = FMOD_THREAD_TYPE_PROFILER,
        /// Thread for processing Studio API commands and scheduling sound playback.
        StudioUpdate     = FMOD_THREAD_TYPE_STUDIO_UPDATE,
        /// Thread for asynchronously loading [studio::Bank] metadata.
        StudioLoadBank   = FMOD_THREAD_TYPE_STUDIO_LOAD_BANK,
        /// Thread for asynchronously loading [studio::Bank] sample data.
        StudioLoadSample = FMOD_THREAD_TYPE_STUDIO_LOAD_SAMPLE,
        /// Thread for processing medium size delay lines for [DspType::ConvolutionReverb].
        Convolution1     = FMOD_THREAD_TYPE_CONVOLUTION1,
        /// Thread for processing larger size delay lines for [DspType::ConvolutionReverb].
        Convolution2     = FMOD_THREAD_TYPE_CONVOLUTION2,
    }
}

fmod_flags! {
    /// Bitfield for specifying the CPU core a given thread runs on.
    ///
    /// The platform agnostic thread groups, A, B and C give recommendations about FMOD threads that should be separated from one another.
    /// Platforms with fixed CPU core counts will try to honor this request, those that don't will leave affinity to the operating system.
    /// See the FMOD platform specific docs for each platform to see how the groups map to cores.
    ///
    /// If an explicit core affinity is given, i.e. [ThreadAffinity::Core11] and that core is unavailable a fatal error will be produced.
    ///
    /// Explicit core assignment up to [ThreadAffinity::Core(61)][Self::Core] is supported for platforms with that many cores.
    pub struct ThreadAffinity: FMOD_THREAD_AFFINITY {
        // Platform agnostic thread groupings
        #[default]
        /// For a given thread use the default listed below, i.e. [ThreadType::Mixer] uses [ThreadAffinity::Mixer].
        GroupDefault = FMOD_THREAD_AFFINITY_GROUP_DEFAULT as i64,
        /// Grouping A is recommended to isolate the mixer thread [ThreadType::Mixer].
        GroupA       = FMOD_THREAD_AFFINITY_GROUP_A as i64,
        /// Grouping B is recommended to isolate the Studio update thread [ThreadType::StudioUpdate].
        GroupB       = FMOD_THREAD_AFFINITY_GROUP_B as i64,
        /// Grouping C is recommended for all remaining threads.
        GroupC       = FMOD_THREAD_AFFINITY_GROUP_C as i64,
        // Thread defaults
        /// Default affinity for [ThreadType::Mixer].
        Mixer            = FMOD_THREAD_AFFINITY_MIXER as i64,
        /// Default affinity for [ThreadType::Feeder].
        Feeder           = FMOD_THREAD_AFFINITY_FEEDER as i64,
        /// Default affinity for [ThreadType::Stream].
        Stream           = FMOD_THREAD_AFFINITY_STREAM as i64,
        /// Default affinity for [ThreadType::File].
        File             = FMOD_THREAD_AFFINITY_FILE as i64,
        /// Default affinity for [ThreadType::NonBlocking].
        NonBlocking      = FMOD_THREAD_AFFINITY_NONBLOCKING as i64,
        /// Default affinity for [ThreadType::Record].
        Record           = FMOD_THREAD_AFFINITY_RECORD as i64,
        /// Default affinity for [ThreadType::Geometry].
        Geometry         = FMOD_THREAD_AFFINITY_GEOMETRY as i64,
        /// Default affinity for [ThreadType::Profiler].
        Profiler         = FMOD_THREAD_AFFINITY_PROFILER as i64,
        /// Default affinity for [ThreadType::StudioUpdate].
        StudioUpdate     = FMOD_THREAD_AFFINITY_STUDIO_UPDATE as i64,
        /// Default affinity for [ThreadType::StudioLoadBank].
        StudioLoadBank   = FMOD_THREAD_AFFINITY_STUDIO_LOAD_BANK as i64,
        /// Default affinity for [ThreadType::StudioLoadSample].
        StudioLoadSample = FMOD_THREAD_AFFINITY_STUDIO_LOAD_SAMPLE as i64,
        /// Default affinity for [ThreadType::Convolution1].
        Convolution1     = FMOD_THREAD_AFFINITY_CONVOLUTION1 as i64,
        /// Default affinity for [ThreadType::Convolution2].
        Convolution2     = FMOD_THREAD_AFFINITY_CONVOLUTION2 as i64,
        // Core mask, valid up to 1 << 61
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
        if n <= 61 {
            ThreadAffinity::from_raw(1 << n)
        } else {
            panic!("thread affinity to core >61 given (nice CPU btw)")
        }
    }
}

fmod_typedef! {
    /// Scheduling priority to assign a given thread to.
    ///
    /// The platform agnostic priorities are used to rank FMOD threads against one another for best runtime scheduling.
    /// Platforms will translate these values in to platform specific priorities.
    /// See the FMOD platform specific docs for each platform to see how the agnostic priorities map to specific values.
    ///
    /// Explicit platform specific priorities can be given within the range of [ThreadPriority::PlatformMin] to [ThreadPriority::PlatformMax].
    /// See platform documentation for details on the available priority values for a given operating system.
    pub enum ThreadPriority: FMOD_THREAD_PRIORITY {
        // Platform specific priority range
        /// Lower bound of platform specific priority range.
        PlatformMin = FMOD_THREAD_PRIORITY_PLATFORM_MIN,
        /// Upper bound of platform specific priority range.
        PlatformMax = FMOD_THREAD_PRIORITY_PLATFORM_MAX as i32,
        // Platform agnostic priorities, maps internally to platform specific value
        #[default]
        /// For a given thread use the default listed below, i.e. [ThreadType::Mixer] uses [ThreadPriority::Mixer].
        Default  = FMOD_THREAD_PRIORITY_DEFAULT,
        /// Low platform agnostic priority.
        Low      = FMOD_THREAD_PRIORITY_LOW,
        /// Medium platform agnostic priority.
        Medium   = FMOD_THREAD_PRIORITY_MEDIUM,
        /// High platform agnostic priority.
        High     = FMOD_THREAD_PRIORITY_HIGH,
        /// Very high platform agnostic priority.
        VeryHigh = FMOD_THREAD_PRIORITY_VERY_HIGH,
        /// Extreme platform agnostic priority.
        Extreme  = FMOD_THREAD_PRIORITY_EXTREME,
        /// Critical platform agnostic priority.
        Critical = FMOD_THREAD_PRIORITY_CRITICAL,
        // Thread defaults
        /// Default priority for [ThreadType::Mixer].
        Mixer            = FMOD_THREAD_PRIORITY_MIXER,
        /// Default priority for [ThreadType::Feeder].
        Feeder           = FMOD_THREAD_PRIORITY_FEEDER,
        /// Default priority for [ThreadType::Stream].
        Stream           = FMOD_THREAD_PRIORITY_STREAM,
        /// Default priority for [ThreadType::File].
        File             = FMOD_THREAD_PRIORITY_FILE,
        /// Default priority for [ThreadType::NonBlocking].
        NonBlocking      = FMOD_THREAD_PRIORITY_NONBLOCKING,
        /// Default priority for [ThreadType::Record].
        Record           = FMOD_THREAD_PRIORITY_RECORD,
        /// Default priority for [ThreadType::Geometry].
        Geometry         = FMOD_THREAD_PRIORITY_GEOMETRY,
        /// Default priority for [ThreadType::Profiler].
        Profiler         = FMOD_THREAD_PRIORITY_PROFILER,
        /// Default priority for [ThreadType::StudioUpdate].
        StudioUpdate     = FMOD_THREAD_PRIORITY_STUDIO_UPDATE,
        /// Default priority for [ThreadType::StudioLoadBank].
        StudioLoadBank   = FMOD_THREAD_PRIORITY_STUDIO_LOAD_BANK,
        /// Default priority for [ThreadType::StudioLoadSample].
        StudioLoadSample = FMOD_THREAD_PRIORITY_STUDIO_LOAD_SAMPLE,
        /// Default priority for [ThreadType::Convolution1].
        Convolution1     = FMOD_THREAD_PRIORITY_CONVOLUTION1,
        /// Default priority for [ThreadType::Convolution2].
        Convolution2     = FMOD_THREAD_PRIORITY_CONVOLUTION2,
    }
}

impl ThreadPriority {
    #[allow(non_snake_case)]
    pub const fn Platform(n: i32) -> ThreadPriority {
        if ThreadPriority::PlatformMin.into_raw() <= n
            && n <= ThreadPriority::PlatformMax.into_raw()
        {
            ThreadPriority::from_raw(n)
        } else {
            panic!("thread priority outside of platform specific range given")
        }
    }
}

fmod_typedef! {
    /// Stack space available to the given thread.
    ///
    /// Stack size can be specified explicitly, however for each thread you should provide a size equal to or larger than the expected default or risk causing a stack overflow at runtime.
    pub enum ThreadStackSize: FMOD_THREAD_STACK_SIZE {
        #[default]
        /// For a given thread use the default listed below, i.e. [ThreadType::Mixer] uses [ThreadStackSize::Mixer].
        Default          = FMOD_THREAD_STACK_SIZE_DEFAULT,
        /// Default stack size for [ThreadType::Mixer].
        Mixer            = FMOD_THREAD_STACK_SIZE_MIXER,
        /// Default stack size for [ThreadType::Feeder].
        Feeder           = FMOD_THREAD_STACK_SIZE_FEEDER,
        /// Default stack size for [ThreadType::Stream].
        Stream           = FMOD_THREAD_STACK_SIZE_STREAM,
        /// Default stack size for [ThreadType::File].
        File             = FMOD_THREAD_STACK_SIZE_FILE,
        /// Default stack size for [ThreadType::NonBlocking].
        NonBlocking      = FMOD_THREAD_STACK_SIZE_NONBLOCKING,
        /// Default stack size for [ThreadType::Record].
        Record           = FMOD_THREAD_STACK_SIZE_RECORD,
        /// Default stack size for [ThreadType::Geometry].
        Geometry         = FMOD_THREAD_STACK_SIZE_GEOMETRY,
        /// Default stack size for [ThreadType::Profiler].
        Profiler         = FMOD_THREAD_STACK_SIZE_PROFILER,
        /// Default stack size for [ThreadType::StudioUpdate].
        StudioUpdate     = FMOD_THREAD_STACK_SIZE_STUDIO_UPDATE,
        /// Default stack size for [ThreadType::StudioLoadBank].
        StudioLoadBank   = FMOD_THREAD_STACK_SIZE_STUDIO_LOAD_BANK,
        /// Default stack size for [ThreadType::StudioLoadSample].
        StudioLoadSample = FMOD_THREAD_STACK_SIZE_STUDIO_LOAD_SAMPLE,
        /// Default stack size for [ThreadType::Convolution1].
        Convolution1     = FMOD_THREAD_STACK_SIZE_CONVOLUTION1,
        /// Default stack size for [ThreadType::Convolution2].
        Convolution2     = FMOD_THREAD_STACK_SIZE_CONVOLUTION2,
    }
}

impl ThreadStackSize {
    #[allow(non_snake_case)]
    pub const fn Explicit(n: u32) -> ThreadStackSize {
        ThreadStackSize::from_raw(n)
    }
}
