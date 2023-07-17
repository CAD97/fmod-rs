use {
    fmod::{raw::*, *},
    parking_lot::RwLockUpgradableReadGuard,
    std::ptr,
};

/// # Lifetime management.
impl System {
    /// Create an instance of the FMOD system.
    ///
    /// Only a single system
    #[cfg_attr(feature = "studio", doc = " (or [studio system][studio::System])")]
    /// can exist safely at a time; further attempts to create a system will
    /// return an error. See [`new_unchecked`][Self::new_unchecked] for more
    /// information about why having multiple systems is unsafe.
    ///
    /// In the common case where the system is used as a global resource, you
    /// can use [`Handle::leak`] to get `&'static System`, which will then allow
    /// all resources aquired from the system to be `Handle<'static, Resource>`.
    /// Dealing in `'static` types avoids the lifetime annotation burden and
    /// unlocks new patterns, like [anymap] backed storage used by many ECSs.
    ///
    /// [anymap]: https://lib.rs/crates/anymap
    pub fn new() -> Result<Handle<'static, Self>> {
        // guard against creating multiple systems
        let system_exists = GLOBAL_SYSTEM_STATE.upgradable_read();
        if *system_exists != 0 {
            whoops!("Only one FMOD system may be created safely. \
                Read the docs on `System::new_unchecked` if you actually mean to create more than one system. \
                Note: constructing a studio system automatically creates a core system for you!");
            yeet!(Error::Initialized);
        }

        // guard against racing other free API calls
        let mut system_count = RwLockUpgradableReadGuard::upgrade(system_exists);

        // actual creation
        unsafe { Self::new_inner(&mut system_count) }
    }

    /// Create an instance of the FMOD system.
    ///
    /// # ⚠ SAFETY WARNING ⚠
    ///
    /// Working with multiple FMOD systems is fraught with unsafety. Creating
    /// and releasing FMOD systems is *thread unsafe*! If creating or releasing
    /// a system potentially races with *any* FMOD API call (including (but not
    /// limited to) other system create/release calls), this is a data race and
    /// potential UB.
    ///
    /// Additionally, FMOD makes no guarantee that using handles with systems
    /// other than the one that created them to correctly cause an error. The
    /// `Studio::System::release` documentation says (as of 2.02.05):
    ///
    /// > All handles or pointers to objects associated with a Studio System
    /// > object become invalid when the Studio System object is released. The
    /// > FMOD Studio API attempts to protect against stale handles and pointers
    /// > being used with a different Studio System object but this protection
    /// > cannot be guaranteed and attempting to use stale handles or pointers
    /// > may cause undefined behavior.
    ///
    /// and it is reasonable to assume that this applies to the core / low-level
    /// API as well. At a minimum, the `multiple_systems.cpp` example says that
    ///
    /// > Note that sounds created on device A cannot be played on device B and
    /// > vice versa.
    ///
    /// but experimental testing in said example has it not return an error.
    /// This implies that such practice is not even protected against for
    /// pointer-like handles, and should be considered UB.
    ///
    /// If you only need a single system, use [`new`][Self::new] instead; it
    /// ensures that only a single system is ever created, and thus no race nor
    /// stale/misused handles can occur. When dropping systems when multiple
    /// systems can be live, you need to ensure that dropping the handle cannot
    /// race with any other FMOD API calls.
    ///
    /// # Safety
    ///
    /// In summary, if you construct multiple systems, you must:
    ///
    /// - Ensure that system creation and releasing does not potentially race
    ///   any FMOD API calls.
    ///   - Note that calling this function makes dropping *any* system handles
    ///     `unsafe`, as that potentially races with any API calls in the other
    ///     live systems!
    /// - Ensure that handles created in one system are not used with a
    ///   different system.
    ///
    /// In short: if you use `new_unchecked`, you're on your own.
    ///
    /// # What would it take to make this safe?
    ///
    /// There is already a global `RWLock` to prevent safe multiple system
    /// creation. Avoiding racing against the rest of the API would thus just
    /// be making every single API call take a read lock. This actually isn't
    /// *that* much of a pessimization, but it's not the only requirement.
    ///
    /// To keep reference handles within the originating system, however,
    /// requires a generative brand, à la [ghost-cell], [qcell's `LCell`], or
    /// [generativity]. This has a notable downside: the system is no longer
    /// `'static`, as it caries around the branded lifetime. This means that
    /// the system can no longer be stored in `'static` storage such as used by
    /// most Rust game engines' resource management flows, even when another
    /// library reencapsulates them.
    ///
    /// The cost of the latter solution was deemed enough that multiple systems,
    /// already being a niche use case, can be relegated to `unsafe` with some
    /// subtle pitfalls. These pitfalls are the same as when using FMOD's API
    /// directly, with the exception of FMOD.rs adding an implicit RAII release.
    ///
    /// If you would like to make the release explicit to avoid the implicit
    /// point of `unsafe`ty, you can [`Handle::leak`] all of your systems, and
    /// then use [`Handle::unleak`] to drop them unsafely.
    ///
    /// [generativity]: https://lib.rs/crates/generativity
    /// [ghost-cell]: https://lib.rs/crates/ghost-cell
    /// [qcell's `LCell`]: https://lib.rs/crates/qcell
    pub unsafe fn new_unchecked() -> Result<Handle<'static, Self>> {
        let mut system_count = GLOBAL_SYSTEM_STATE.write();
        Self::new_inner(&mut system_count)
    }

    unsafe fn new_inner(system_count: &mut usize) -> Result<Handle<'static, Self>> {
        debug::initialize_default(); // setup debug logging

        let mut raw = ptr::null_mut();
        ffi!(FMOD_System_Create(&mut raw, FMOD_VERSION))?;
        *system_count += 1;
        Ok(Handle::new(raw))
    }

    /// Initialize the system object and prepare FMOD for playback.
    ///
    /// Most API functions require an initialized System object before they will
    /// succeed, otherwise they will return [Error::Uninitialized]. Some can
    /// only be called before initialization. These are:
    ///
    /// - [Memory_Initialize]
    /// - [System::set_software_format]
    /// - [System::set_software_channels]
    /// - [System::set_dsp_buffer_size]
    ///
    /// [System::set_output] / [System::set_output_by_plugin] can be called
    /// before or after [System::init] on Android, GameCore, UWP, Windows and
    /// Mac. Other platforms can only call this **before** [System::init].
    ///
    /// `max_channels` is the maximum number of [Channel] objects available for
    /// playback, also known as virtual channels. Virtual channels will play
    /// with minimal overhead, with a subset of 'real' voices that are mixed,
    /// and selected based on priority and audibility. See the [Virtual Voices]
    /// guide for more information.
    ///
    /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
    pub fn init(&self, max_channels: i32, flags: InitFlags) -> Result {
        // I hope FMOD does the right thing for a nullptr driver data in all cases...
        unsafe { self.init_ex(max_channels, flags, ptr::null()) }
    }

    /// Initialize the system object and prepare FMOD for playback.
    ///
    /// # Safety
    ///
    /// `extra_driver_data` must be correct. It represents additional output
    /// specific initialization data. This will be passed to the output plugin.
    /// See [OutputType] for descriptions of data that can be passed in, based
    /// on the selected output mode.
    pub unsafe fn init_ex(
        &self,
        max_channels: i32,
        flags: InitFlags,
        extra_driver_data: *const (),
    ) -> Result {
        let flags = InitFlags::into_raw(flags);
        ffi!(FMOD_System_Init(
            self.as_raw(),
            max_channels,
            flags,
            extra_driver_data as *mut _,
        ))?;
        Ok(())
    }

    // TODO: safe init_ex wrappers for WavWriter[Nrt], PulseAudio

    /// Close the connection to the output and return to an uninitialized state
    /// without releasing the object.
    ///
    /// All pre-initialize configuration settings will remain and the System can
    /// be reinitialized as needed.
    ///
    /// # Safety
    ///
    /// Closing renders objects created with this System invalid. Make sure any
    /// Sound, ChannelGroup, Geometry and DSP objects are released before
    /// calling this.
    pub unsafe fn close(&self) -> Result {
        ffi!(FMOD_System_Close(self.as_raw()))?;
        Ok(())
    }

    raw! {
        /// Closes and frees this object and its resources.
        ///
        /// This will internally call [`System::close`], so calling
        /// [`System::close`] before this function is not necessary.
        pub unsafe fn raw_release(raw: *mut FMOD_SYSTEM) -> FMOD_RESULT {
            let mut system_count = GLOBAL_SYSTEM_STATE.write();
            let result = FMOD_System_Release(raw);
            if result == FMOD_OK {
                *system_count -= 1;
                FMOD_OK
            } else {
                result
            }
        }
    }

    /// Updates the FMOD system.
    ///
    /// Should be called once per 'game' tick, or once per frame in your
    /// application to perform actions such as:
    ///
    /// - Panning and reverb from 3D attributes changes.
    /// - Virtualization of [Channel]s based on their audibility.
    /// - Mixing for non-realtime output types. See comment below.
    /// - Streaming if using [InitFlags::StreamFromUpdate].
    /// - Mixing if using [InitFlags::MixFromUpdate].
    /// - Firing callbacks that are deferred until Update.
    ///
    /// - DSP cleanup.
    ///
    /// If [OutputType::NoSoundNrt] or [OutputType::WavWriterNrt] output modes
    /// are used, this function also drives the software / DSP engine, instead
    /// of it running asynchronously in a thread as is the default behavior.  
    /// This can be used for faster than realtime updates to the decoding or
    /// DSP engine which might be useful if the output is the wav writer for
    /// example.
    ///
    /// If [InitFlags::StreamFromUpdate] is used, this function will update the
    /// stream engine. Combining this with the non realtime output will mean
    /// smoother captured output.
    pub fn update(&self) -> Result {
        ffi!(FMOD_System_Update(self.as_raw()))?;
        Ok(())
    }

    /// Suspend mixer thread and relinquish usage of audio hardware while
    /// maintaining internal state.
    ///
    /// Used on mobile platforms when entering a backgrounded state to reduce
    /// CPU to 0%.
    ///
    /// All internal state will be maintained, i.e. created sound and channels
    /// will stay available in memory.
    ///
    /// # Safety
    ///
    /// No FMOD API calls may be made until [System::mixer_resume] is called.
    pub unsafe fn mixer_suspend(&self) -> Result {
        ffi!(FMOD_System_MixerSuspend(self.as_raw()))?;
        Ok(())
    }

    /// Resume mixer thread and reacquire access to audio hardware.
    ///
    /// Used on mobile platforms when entering the foreground after being
    /// suspended.
    ///
    /// All internal state will resume, i.e. created sound and channels are
    /// still valid and playback will continue.
    ///
    /// HTML5 specific: Used to start audio from a user interaction event, like
    /// a mouse click or screen touch event. Without this call audio may not
    /// start on some browsers.
    ///
    /// # Safety
    ///
    /// Must be called on the same thread as [System::mixer_suspend].
    pub unsafe fn mixer_resume(&self) -> Result {
        ffi!(FMOD_System_MixerResume(self.as_raw()))?;
        Ok(())
    }
}

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
}
