use fmod::{raw::*, *};

flags! {
    /// Specify the requested information to be output when using the logging version of FMOD.
    pub struct DebugFlags: FMOD_DEBUG_FLAGS {
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
    pub struct MemoryType: FMOD_MEMORY_TYPE {
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
    pub struct InitFlags: FMOD_INITFLAGS {
        #[default]
        /// Initialize normally
        Normal                 = FMOD_INIT_NORMAL,
        /// No stream thread is created internally. Streams are driven from [System::update]. Mainly used with non-realtime outputs.
        StreamFromUpdate       = FMOD_INIT_STREAM_FROM_UPDATE,
        /// No mixer thread is created internally. Mixing is driven from [System::update]. Only applies to polling based output modes such as [OutputType::NoSound], [OutputType::WavWriter].
        MixFromUpdate          = FMOD_INIT_MIX_FROM_UPDATE,
        /// 3D calculations will be performed in right-handed coordinates.
        RightHanded3d          = FMOD_INIT_3D_RIGHTHANDED,
        /// Enables hard clipping of output values greater than 1.0 or less than -1.0.
        ClipOutput             = FMOD_INIT_CLIP_OUTPUT,
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
    pub struct SystemCallbackType: FMOD_SYSTEM_CALLBACK_TYPE {
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
        /// Called from the mixer thread when the System record position changed.
        RecordPositionChanged  = FMOD_SYSTEM_CALLBACK_RECORDLISTCHANGED,
        /// Mask representing all callback types.
        All                    = FMOD_SYSTEM_CALLBACK_ALL,
    }

    /// Sound description bitfields, bitwise OR them together for loading and describing sounds.
    ///
    /// By default a sound will open as a static sound that is decompressed fully into memory to PCM. (ie equivalent of [Mode::CreateSample]) To have a sound stream instead, use [Mode::CreateStream], or use the wrapper function [System::create_stream].
    ///
    /// Some opening modes (ie [Mode::OpenUser], [Mode::OpenMemory], [Mode::OpenMemoryPoint], [Mode::OpenRaw]) will need extra information. This can be provided using the [CreateSoundExInfo] structure.
    ///
    /// Specifying [Mode::OpenMemoryPoint] will POINT to your memory rather allocating its own sound buffers and duplicating it internally. This means you cannot free the memory while FMOD is using it, until after the sound is released.
    ///
    /// With [Mode::OpemMemoryPoint], for PCM formats, only WAV, FSB, and RAW are supported. For compressed formats, only those formats supported by [Mode::CreateCompressedSample] are supported.
    ///
    /// With [Mode::OpenMemoryPoint] and [Mode::OpenRaw] or PCM, if using them together, note that you must pad the data on each side by 16 bytes. This is so fmod can modify the ends of the data for looping / interpolation / mixing purposes. If a wav file, you will need to insert silence, and then reset loop points to stop the playback from playing that silence.
    pub struct Mode: FMOD_MODE {
        #[default]
        /// Default for all modes listed below. [Mode::LoopOff], [Mode::D2], [Mode::WorldRelative3d], [Mode::InverseRolloff3d]
        Default                 = FMOD_DEFAULT,
        /// For non looping sounds. (DEFAULT). Overrides [Mode::LoopNormal] / [Mode::LoopBidi].
        LoopOff                 = FMOD_LOOP_OFF,
        /// For forward looping sounds.
        LoopNormal              = FMOD_LOOP_NORMAL,
        /// For bidirectional looping sounds. (only works on software mixed static sounds).
        LoopBidi                = FMOD_LOOP_BIDI,
        /// Ignores any 3d processing. (DEFAULT).
        D2                      = FMOD_2D,
        /// Makes the sound positionable in 3D. Overrides [Mode::D3].
        D3                      = FMOD_3D,
        /// Decompress at runtime, streaming from the source provided (ie from disk). Overrides [Mode::CreateSample] and [Mode::CreateCompressedSample]. Note a stream can only be played once at a time due to a stream only having 1 stream buffer and file handle. Open multiple streams to have them play concurrently.
        CreateStream            = FMOD_CREATESTREAM,
        /// Decompress at loadtime, decompressing or decoding whole file into memory as the target sample format (ie PCM). Fastest for playback and most flexible.
        CreateSample            = FMOD_CREATESAMPLE,
        /// Load MP2/MP3/FADPCM/IMAADPCM/Vorbis/AT9 or XMA into memory and leave it compressed. Vorbis/AT9/FADPCM encoding only supported in the .FSB container format. During playback the FMOD software mixer will decode it in realtime as a 'compressed sample'. Overrides [Mode::CreateSample]. If the sound data is not one of the supported formats, it will behave as if it was created with [Mode::CreateSample] and decode the sound into PCM.
        CreateCompressedSample  = FMOD_CREATECOMPRESSEDSAMPLE,
        /// Opens a user created static sample or stream. Use [CreateSoundExInfo] to specify format, defaultfrequency, numchannels, and optionally a read callback. If a user created 'sample' is created with no read callback, the sample will be empty. Use [Sound::lock] and [Sound::unlock] to place sound data into the sound if this is the case.
        OpenUser                = FMOD_OPENUSER,
        /// "name_or_data" will be interpreted as a pointer to memory instead of filename for creating sounds. Use [CreateSoundExInfo] to specify length. If used with [Mode::CreateSample] or [Mode::CreateCompressedSample], FMOD duplicates the memory into its own buffers. Your own buffer can be freed after open, unless you are using [Mode::NonBlocking] then wait until the Sound is in the [OpenState::Ready] state. If used with [Mode::CreateStream], FMOD will stream out of the buffer whose pointer you passed in. In this case, your own buffer should not be freed until you have finished with and released the stream.
        OpenMemory              = FMOD_OPENMEMORY,
        /// "name_or_data" will be interpreted as a pointer to memory instead of filename for creating sounds. Use [CreateSoundExInfo] to specify length. This differs to [Mode::OpenMemory] in that it uses the memory as is, without duplicating the memory into its own buffers. Cannot be freed after open, only after the sound is released. Will not work if the data is compressed and [Mode::CreateCompressedSample] is not used. Cannot be used in conjunction with [CreateSoundExInfo::encryption_key].
        OpenMemoryPoint         = FMOD_OPENMEMORY_POINT,
        /// Will ignore file format and treat as raw pcm. Use [CreateSoundExInfo] to specify format. Requires at least default_frequency, num_channels and format to be specified before it will open. Must be little endian data.
        OpenRaw                 = FMOD_OPENRAW,
        /// Just open the file, dont prebuffer or read. Good for fast opens for info, or when [Sound::read_data] is to be used.
        OpenOnly                = FMOD_OPENONLY,
        /// For [System::create_sound] - for accurate [Sound::get_length] / [Channel::set_position] on VBR MP3, and MOD/S3M/XM/IT/MIDI files. Scans file first, so takes longer to open. [Mode::OpenOnly] does not affect this.
        AccurateTime            = FMOD_ACCURATETIME,
        /// For corrupted / bad MP3 files. This will search all the way through the file until it hits a valid MPEG header. Normally only searches for 4k.
        MpegSearch              = FMOD_MPEGSEARCH,
        /// For opening sounds and getting streamed subsounds (seeking) asynchronously. Use [Sound::get_open_state] to poll the state of the sound as it opens or retrieves the subsound in the background.
        NonBlocking             = FMOD_NONBLOCKING,
        /// Unique sound, can only be played one at a time
        Unique                  = FMOD_UNIQUE,
        /// Make the sound's position, velocity and orientation relative to the listener.
        HeadRelative3d          = FMOD_3D_HEADRELATIVE,
        /// Make the sound's position, velocity and orientation absolute (relative to the world). (DEFAULT)
        WorldRelative3d         = FMOD_3D_WORLDRELATIVE,
        /// This sound will follow the inverse rolloff model where mindistance = full volume, maxdistance = where sound stops attenuating, and rolloff is fixed according to the global rolloff factor. (DEFAULT)
        InverseRolloff3d        = FMOD_3D_INVERSEROLLOFF,
        /// This sound will follow a linear rolloff model where mindistance = full volume, maxdistance = silence.
        LinearRolloff3d         = FMOD_3D_LINEARROLLOFF,
        /// This sound will follow a linear-square rolloff model where mindistance = full volume, maxdistance = silence.
        LinearSquareRolloff3d   = FMOD_3D_LINEARSQUAREROLLOFF,
        /// This sound will follow the inverse rolloff model at distances close to mindistance and a linear-square rolloff close to maxdistance.
        InverseTaperedRolloff3d = FMOD_3D_INVERSETAPEREDROLLOFF,
        /// This sound will follow a rolloff model defined by [Sound::set_3d_custom_rolloff] / [Channel::set_3d_custom_rolloff].
        CustomRolloff3d         = FMOD_3D_CUSTOMROLLOFF,
        /// Is not affect by geometry occlusion. If not specified in [Sound::set_mode], or [Channel::set_mode], the flag is cleared and it is affected by geometry again.
        IgnoreGeometry3d        = FMOD_3D_IGNOREGEOMETRY,
        /// Skips id3v2/asf/etc tag checks when opening a sound, to reduce seek/read overhead when opening files.
        IgnoreTags              = FMOD_IGNORETAGS,
        /// Removes some features from samples to give a lower memory overhead, like [Sound::get_name].
        LowMem                  = FMOD_LOWMEM,
        /// For sounds that start virtual (due to being quiet or low importance), instead of swapping back to audible, and playing at the correct offset according to time, this flag makes the sound play from the start.
        VirtualPlayFromStart    = FMOD_VIRTUAL_PLAYFROMSTART,
    }

    /// Flags that describe the speakers present in a given signal.
    pub struct ChannelMask: FMOD_CHANNELMASK {
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

    /// Output type specific index for when there are multiple instances of a port type.
    pub struct PortIndex: FMOD_PORT_INDEX {
        /// Use when a port index is not required
        None = FMOD_PORT_INDEX_NONE as _,
        /// Use as a flag to indicate the intended controller is associated with a VR headset
        VrController = FMOD_PORT_INDEX_FLAG_VR_CONTROLLER as _,
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
