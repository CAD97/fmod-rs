use fmod::{raw::*, *};

flags! {
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
}

flags! {
    /// Output type specific index for when there are multiple instances of a port type.
    pub struct PortIndex: FMOD_PORT_INDEX {
        /// Use when a port index is not required
        None = FMOD_PORT_INDEX_NONE as _,
        /// Use as a flag to indicate the intended controller is associated with a VR headset
        VrController = FMOD_PORT_INDEX_FLAG_VR_CONTROLLER as _,
    }
}
