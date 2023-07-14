use crate::utils::catch_user_unwind;

use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    parking_lot::RwLockUpgradableReadGuard,
    smart_default::SmartDefault,
    std::{
        borrow::Cow,
        ffi::{c_char, c_void, CStr},
        marker::PhantomData,
        mem,
        mem::ManuallyDrop,
        ptr, slice,
        time::Duration,
    },
};

opaque! {
    /// Management object from which all resources are created and played.
    class System = FMOD_SYSTEM, FMOD_System_* (System::raw_release);
}

// -------------------------------------------------------------------------------------------------

/// # Lifetime management
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

    unsafe fn raw_release(raw: *mut FMOD_SYSTEM) -> FMOD_RESULT {
        let mut system_count = GLOBAL_SYSTEM_STATE.write();
        let result = FMOD_System_Release(raw);
        if result == FMOD_OK {
            *system_count -= 1;
            FMOD_OK
        } else {
            result
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

// -------------------------------------------------------------------------------------------------

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

// -------------------------------------------------------------------------------------------------

/// # Device selection.
impl System {
    /// Sets the type of output interface used to run the mixer.
    ///
    /// This function is typically used to select between different OS specific
    /// audio APIs which may have different features.
    ///
    /// It is only necessary to call this function if you want to specifically
    /// switch away from the default output mode for the operating system. The
    /// most optimal mode is selected by default for the operating system.
    ///
    /// (Windows, UWP, GameCore, Android, MacOS, iOS, Linux Only) This function
    /// can be called after System::init to perform special handling of driver
    /// disconnections, see [SystemCallback::DeviceListChanged].
    pub fn set_output(&self, output: fmod::OutputType) -> Result {
        let output = output.into_raw();
        ffi!(FMOD_System_SetOutput(self.as_raw(), output))?;
        Ok(())
    }

    /// Retrieves the type of output interface used to run the mixer.
    pub fn get_output(&self) -> Result<fmod::OutputType> {
        let mut output = OutputType::zeroed();
        ffi!(FMOD_System_GetOutput(self.as_raw(), output.as_raw_mut()))?;
        Ok(output)
    }

    /// Retrieves the number of output drivers available for the selected output
    /// type.
    ///
    /// If [System::set_output] has not been called, this function will return
    /// the number of drivers available for the default output type.
    /// A possible use for this function is to iterate through available sound
    /// devices for the current output type, and use [System::get_driver_name]
    /// to get the device's name and [System::get_driver_info] for other
    /// attributes.
    pub fn get_num_drivers(&self) -> Result<i32> {
        let mut numdrivers = 0;
        ffi!(FMOD_System_GetNumDrivers(self.as_raw(), &mut numdrivers))?;
        Ok(numdrivers)
    }

    // NB: we split get_driver_info/name into separate calls for two reasons:
    // getting *just* the name is common, and the name has extra retry
    // requirements to validate non-truncation and UTF-8. This does, however,
    // mean that getting all of the driver info requires an extra FFI call.

    /// Retrieves identification information about a sound device specified by
    /// its index, and specific to the selected output mode.
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[0, System::get_num_drivers]</dd>
    /// </dl>
    pub fn get_driver_info(&self, id: i32) -> Result<DriverInfo> {
        let mut guid = Guid::default();
        let mut system_rate = 0;
        let mut speaker_mode = SpeakerMode::default();
        let mut speaker_mode_channels = 0;

        ffi!(FMOD_System_GetDriverInfo(
            self.as_raw(),
            id,
            ptr::null_mut(),
            0,
            guid.as_raw_mut(),
            &mut system_rate,
            speaker_mode.as_raw_mut(),
            &mut speaker_mode_channels,
        ))?;

        Ok(DriverInfo {
            guid,
            system_rate,
            speaker_mode,
            speaker_mode_channels,
            state: DriverState::default(),
        })
    }

    /// Retrieves the name of a sound device specified by its index, specific to
    /// the selected output mode.
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[0, System::get_num_drivers]</dd>
    /// </dl>
    pub fn get_driver_name(&self, id: i32, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_System_GetDriverInfo(
                    self.as_raw(),
                    id,
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ))
            })
        }
    }

    /// Sets the output driver for the selected output type.
    ///
    /// When an output type has more than one driver available, this function
    /// can be used to select between them.
    ///
    /// If this function is called after [System::init], the current driver will
    /// be shutdown and the newly selected driver will be initialized / started.
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[0, System::get_num_drivers]</dd>
    /// </dl>
    pub fn set_driver(&self, id: i32) -> Result {
        ffi!(FMOD_System_SetDriver(self.as_raw(), id))?;
        Ok(())
    }

    /// Retrieves the output driver index for the selected output type.
    ///
    /// 0 represents the default for the output type.
    pub fn get_driver(&self) -> Result<i32> {
        let mut driver = 0;
        ffi!(FMOD_System_GetDriver(self.as_raw(), &mut driver))?;
        Ok(driver)
    }
}

// -------------------------------------------------------------------------------------------------

enum_struct! {
    /// Built-in output types that can be used to run the mixer.
    ///
    /// To pass information to the driver when initializing use the `extra_driver_data` parameter in [System::init_ex] for the following reasons:
    ///
    /// - [OutputType::WavWriter] - `*const c_char` file name that the wav writer will output to.
    /// - [OutputType::WavWriterNrt] - `*const c_char` file name that the wav writer will output to.
    /// - [OutputType::PulseAudio] - `*const c_char` application name to display in OS audio mixer.
    /// - [OutputType::Asio] - `*mut c_void` application window handle.
    ///
    /// Currently these are the only FMOD drivers that take extra information. Other unknown plugins may have different requirements.
    ///
    /// If [OutputType::WavWriterNrt] or [OutputType::NoSoundNrt] are used, and if the [System::update] function is being called very quickly (ie for a non realtime decode) it may be being called too quickly for the FMOD streamer thread to respond to. The result will be a skipping/stuttering output in the captured audio. To remedy this, disable the FMOD streamer thread, and use [InitFlags::StreamFromUpdate] to avoid skipping in the output stream, as it will lock the mixer and the streamer together in the same thread.
    pub enum OutputType: FMOD_OUTPUTTYPE {
        /// Picks the best output mode for the platform. This is the default.
        AutoDetect   = FMOD_OUTPUTTYPE_AUTODETECT,
        /// All - 3rd party plugin, unknown. This is for use with [System::get_output] only.
        Unknown      = FMOD_OUTPUTTYPE_UNKNOWN,
        /// All - Perform all mixing but discard the final output.
        NoSound      = FMOD_OUTPUTTYPE_NOSOUND,
        /// All - Writes output to a .wav file.
        WavWriter    = FMOD_OUTPUTTYPE_WAVWRITER,
        /// All - Non-realtime version of [OutputType::NoSound], one mix per [System::update].
        NoSoundNrt   = FMOD_OUTPUTTYPE_NOSOUND_NRT,
        /// All - Non-realtime version of [OutputType::WavWriter], one mix per [System::update].
        WavWriterNrt = FMOD_OUTPUTTYPE_WAVWRITER_NRT,
        /// Win / UWP / Xbox One / Game Core - Windows Audio Session API. (Default on Windows, Xbox One, Game Core and UWP)
        Wasapi       = FMOD_OUTPUTTYPE_WASAPI,
        /// Win - Low latency ASIO 2.0.
        Asio         = FMOD_OUTPUTTYPE_ASIO,
        /// Linux - Pulse Audio. (Default on Linux if available)
        PulseAudio   = FMOD_OUTPUTTYPE_PULSEAUDIO,
        /// Linux - Advanced Linux Sound Architecture. (Default on Linux if PulseAudio isn't available)
        Alsa         = FMOD_OUTPUTTYPE_ALSA,
        /// Mac / iOS - Core Audio. (Default on Mac and iOS)
        CoreAudio    = FMOD_OUTPUTTYPE_COREAUDIO,
        /// Android - Java Audio Track. (Default on Android 2.2 and below)
        AudioTrack   = FMOD_OUTPUTTYPE_AUDIOTRACK,
        /// Android - OpenSL ES. (Default on Android 2.3 up to 7.1)
        OpenSl       = FMOD_OUTPUTTYPE_OPENSL,
        /// PS4 / PS5 - Audio Out. (Default on PS4, PS5)
        AudioOut     = FMOD_OUTPUTTYPE_AUDIOOUT,
        /// PS4 - Audio3D.
        Audio3d      = FMOD_OUTPUTTYPE_AUDIO3D,
        /// HTML5 - Web Audio ScriptProcessorNode output. (Default on HTML5 if AudioWorkletNode isn't available)
        WebAudio     = FMOD_OUTPUTTYPE_WEBAUDIO,
        /// Switch - nn::audio. (Default on Switch)
        NnAudio      = FMOD_OUTPUTTYPE_NNAUDIO,
        /// Win10 / Xbox One / Game Core - Windows Sonic.
        Winsonic     = FMOD_OUTPUTTYPE_WINSONIC,
        /// Android - AAudio. (Default on Android 8.1 and above)
        AAudio       = FMOD_OUTPUTTYPE_AAUDIO,
        /// HTML5 - Web Audio AudioWorkletNode output. (Default on HTML5 if available)
        AudioWorklet = FMOD_OUTPUTTYPE_AUDIOWORKLET,
        /// Mac / iOS - PHASE framework. (Disabled)
        Phase        = FMOD_OUTPUTTYPE_PHASE,
    }
}

// -------------------------------------------------------------------------------------------------

/// Identification information about a sound device.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DriverInfo {
    /// GUID that uniquely identifies the device.
    pub guid: Guid,
    /// Sample rate this device operates at.
    pub system_rate: i32,
    /// Speaker setup this device is currently using.
    pub speaker_mode: SpeakerMode,
    /// Number of channels in the current speaker setup.
    pub speaker_mode_channels: i32,
    /// Flags that provide additional information about the driver.
    /// Only set for record drivers.
    pub state: DriverState,
}

// -------------------------------------------------------------------------------------------------

/// # Setup.
impl System {
    /// Sets the maximum number of software mixed channels possible.
    ///
    /// This function cannot be called after FMOD is already activated, it must
    /// be called before [System::init], or after [System::close].
    ///
    /// <dl>
    /// <dt>Default</dt><dd>64</dd>
    /// </dl>
    pub fn set_software_channels(&self, num_software_channels: i32) -> Result {
        ffi!(FMOD_System_SetSoftwareChannels(
            self.as_raw(),
            num_software_channels,
        ))?;
        Ok(())
    }

    /// Retrieves the maximum number of software mixed channels possible.
    pub fn get_software_channels(&self) -> Result<i32> {
        let mut num_software_channels = 0;
        ffi!(FMOD_System_GetSoftwareChannels(
            self.as_raw(),
            &mut num_software_channels,
        ))?;
        Ok(num_software_channels)
    }

    /// Sets the output format for the software mixer.
    ///
    /// If loading Studio banks, this must be called with `speaker_mode`
    /// corresponding to the project output format if there is a possibility of
    /// the output audio device not matching the project format. Any differences
    /// between the project format and speakermode will cause the mix to sound
    /// wrong.
    ///
    /// By default `speaker_mode` will assume the setup the OS/output prefers.
    ///
    /// Altering the `sample_rate` from the OS/output preferred rate may incur
    /// extra latency. Altering the `speaker_mode` from the OS/output preferred
    /// mode may cause an upmix/downmix which can alter the sound.
    ///
    /// On lower power platforms such as mobile samplerate will default to 24KHz
    /// to reduce CPU cost.
    ///
    /// This function must be called before [System::init], or after
    /// [System::close].
    pub fn set_software_format(&self, format: SoftwareFormat) -> Result {
        let SoftwareFormat {
            sample_rate,
            speaker_mode,
            num_raw_speakers,
        } = format;
        ffi!(FMOD_System_SetSoftwareFormat(
            self.as_raw(),
            sample_rate,
            speaker_mode.into_raw(),
            num_raw_speakers,
        ))?;
        Ok(())
    }

    /// Retrieves the output format for the software mixer.
    pub fn get_software_format(&self) -> Result<SoftwareFormat> {
        let mut sample_rate = 0;
        let mut speaker_mode = SpeakerMode::default();
        let mut num_raw_speakers = 0;
        ffi!(FMOD_System_GetSoftwareFormat(
            self.as_raw(),
            &mut sample_rate,
            speaker_mode.as_raw_mut(),
            &mut num_raw_speakers
        ))?;
        Ok(SoftwareFormat {
            sample_rate,
            speaker_mode,
            num_raw_speakers,
        })
    }

    /// Sets the buffer size for the FMOD software mixing engine.
    ///
    /// This function is used if you need to control mixer latency or
    /// granularity. Smaller buffersizes lead to smaller latency, but can lead
    /// to stuttering/skipping/unstable sound on slower machines or soundcards
    /// with bad drivers.
    ///
    /// To get the `buffer_length` in milliseconds, divide it by the output rate
    /// and multiply the result by 1000. For a `buffer_length` of 1024 and an
    /// output rate of 48khz (see [System::set_software_format]), milliseconds =
    /// 1024 / 48000 * 1000 = 21.33ms. This means the mixer updates every
    /// 21.33ms.
    ///
    /// To get the total buffer size multiply the `buffer_length` by the
    /// `num_buffers` value. By default this would be 41024 = 4096 samples, or
    /// 421.33ms = 85.33ms. This would generally be the total latency of the
    /// software mixer, but in reality due to one of the buffers being written
    /// to constantly, and the cursor position of the buffer that is audible,
    /// the latency is typically more like the (number of buffers - 1.5)
    /// multiplied by the buffer length.
    ///
    /// To convert from milliseconds back to 'samples', simply multiply the
    /// value in milliseconds by the sample rate of the output (ie 48000 if that
    /// is what it is set to), then divide by 1000.
    ///
    /// The FMOD software mixer mixes to a ringbuffer. The size of this
    /// ringbuffer is determined here. It mixes a block of sound data every
    /// 'bufferlength' number of samples, and there are 'numbuffers' number of
    /// these blocks that make up the entire ringbuffer. Adjusting these values
    /// can lead to extremely low latency performance (smaller values), or
    /// greater stability in sound output (larger values).
    ///
    /// Warning! The 'buffersize' is generally best left alone. Making the
    /// granularity smaller will just increase CPU usage (cache misses and DSP
    /// network overhead). Making it larger affects how often you hear commands
    /// update such as volume/pitch/pan changes. Anything above 20ms will be
    /// noticeable and sound parameter changes will be obvious instead of
    /// smooth.
    ///
    /// FMOD chooses the most optimal size by default for best stability,
    /// depending on the output type. It is not recommended changing this value
    /// unless you really need to. You may get worse performance than the
    /// default settings chosen by FMOD. If you do set the size manually, the
    /// bufferlength argument must be a multiple of four, typically 256, 480,
    /// 512, 1024 or 2048 depedning on your latency requirements.
    ///
    /// The values in milliseconds and average latency expected from the
    /// settings can be calculated using the following code:
    ///
    /// ```rust,ignore
    /// let DspBufferSize { buffer_length, num_buffers } = system.get_dsp_buffer_size()?;
    /// let SoftwareFormat { sample_rate, .. } = system.get_software_format()?;
    ///
    /// let ms = buffer_size.buffer_length as f32 * 1000.0 / software_format.sample_rate as f32;
    ///
    /// println!("Mixer blocksize        = {:.02}", ms);
    /// println!("Mixer Total buffersize = {:.02}", ms * num_buffers);
    /// println!("Mixer Average Latency  = {:.02}", ms * (num_buffers as f32 - 1.5));
    /// ```
    pub fn set_dsp_buffer_size(&self, buffer_size: DspBufferSize) -> Result {
        let DspBufferSize {
            buffer_length,
            num_buffers,
        } = buffer_size;
        ffi!(FMOD_System_SetDSPBufferSize(
            self.as_raw(),
            buffer_length,
            num_buffers,
        ))?;
        Ok(())
    }

    /// Retrieves the buffer size settings for the FMOD software mixing engine.
    ///
    /// To get the `buffer_length` in milliseconds, divide it by the output rate
    /// and multiply the result by 1000. For a `buffer_length` of 1024 and an
    /// output rate of 48khz (see [System::set_software_format]), milliseconds =
    /// 1024 / 48000 * 1000 = 21.33ms. This means the mixer updates every
    /// 21.33ms.
    ///
    /// To get the total buffer size multiply the `buffer_length` by the
    /// `num_buffers` value. By default this would be 41024 = 4096 samples, or
    /// 421.33ms = 85.33ms. This would generally be the total latency of the
    /// software mixer, but in reality due to one of the buffers being written
    /// to constantly, and the cursor position of the buffer that is audible,
    /// the latency is typically more like the (number of buffers - 1.5)
    /// multiplied by the buffer length.
    ///
    /// To convert from milliseconds back to 'samples', simply multiply the
    /// value in milliseconds by the sample rate of the output (ie 48000 if that
    /// is what it is set to), then divide by 1000.
    pub fn get_dsp_buffer_size(&self) -> Result<(u32, i32)> {
        let mut bufferlength = 0;
        let mut numbuffers = 0;
        ffi!(FMOD_System_GetDSPBufferSize(
            self.as_raw(),
            &mut bufferlength,
            &mut numbuffers,
        ))?;
        Ok((bufferlength, numbuffers))
    }

    /// Sets the default file buffer size for newly opened streams.
    ///
    /// Valid units are [TimeUnit::Ms], [Pcm](TimeUnit::Pcm),
    /// [PcmBytes](TimeUnit::PcmBytes), and [RawBytes](TimeUnit::RawBytes).
    ///
    /// The default value is 16384 [TimeUnit::RawBytes]. Larger values will
    /// consume more memory, whereas smaller values may cause buffer under-run /
    /// starvation / stuttering caused by large delays in disk access (ie
    /// netstream), or CPU usage in slow machines, or by trying to play too many
    /// streams at once.
    ///
    /// Does not affect streams created with [Mode::OpenUser], as the buffer
    /// size is specified in [System::create_sound_ex].
    ///
    /// Does not affect latency of playback. All streams are pre-buffered
    /// (unless opened with [Mode::OpenOnly]), so they will always start
    /// immediately.
    ///
    /// Seek and Play operations can sometimes cause a reflush of this buffer.
    ///
    /// If [TimeUnit::RawBytes] is used, the memory allocated is two times the
    /// size passed in, because fmod allocates a double buffer.
    ///
    /// If [TimeUnit::Ms], [TimeUnit::Pcm] or [TimeUnit::PcmBytes] is used, and
    /// the stream is infinite (such as a shoutcast netstream), or VBR, then
    /// FMOD cannot calculate an accurate compression ratio to work with when
    /// the file is opened. This means it will then base the buffersize on
    /// [TimeUnit::PcmBytes], or in other words the number of PCM bytes, but
    /// this will be incorrect for some compressed formats. Use
    /// [TimeUnit::RawBytes] for these type (infinite / undetermined length) of
    /// streams for more accurate read sizes.
    ///
    /// To determine the actual memory usage of a stream, including sound buffer
    /// and other overhead, use [memory::get_stats] before and after creating a
    /// sound.
    ///
    /// Stream may still stutter if the codec uses a large amount of cpu time,
    /// which impacts the smaller, internal 'decode' buffer. The decode buffer
    /// size is changeable via [CreateSoundExInfo].
    pub fn set_stream_buffer_size(&self, file_buffer_size: Time) -> Result {
        ffi!(FMOD_System_SetStreamBufferSize(
            self.as_raw(),
            file_buffer_size.value,
            file_buffer_size.unit.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the default file buffer size for newly opened streams.
    ///
    /// Valid units are [TimeUnit::Ms], [Pcm](TimeUnit::Pcm),
    /// [PcmBytes](TimeUnit::PcmBytes), and [RawBytes](TimeUnit::RawBytes).
    pub fn get_stream_buffer_size(&self) -> Result<(u32, TimeUnit)> {
        let mut file_buffer_size = 0;
        let mut file_buffer_size_type = TimeUnit::zeroed();
        ffi!(FMOD_System_GetStreamBufferSize(
            self.as_raw(),
            &mut file_buffer_size,
            file_buffer_size_type.as_raw_mut(),
        ))?;
        Ok((file_buffer_size, file_buffer_size_type))
    }

    /// Sets advanced settings for the system object, typically to allow
    /// adjusting of settings related to resource usage or audio quality.
    pub fn set_advanced_settings(&self, mut advanced_settings: AdvancedSettings) -> Result {
        ffi!(FMOD_System_SetAdvancedSettings(
            self.as_raw(),
            advanced_settings.as_raw_mut(),
        ))?;
        Ok(())
    }

    /// Retrieves the advanced settings for the system object.
    pub fn get_advanced_settings(&self) -> Result<AdvancedSettings> {
        let mut advanced_settings = AdvancedSettings::default();
        ffi!(FMOD_System_GetAdvancedSettings(
            self.as_raw(),
            advanced_settings.as_raw_mut(),
        ))?;
        Ok(advanced_settings)
    }

    /// Sets the position of the specified speaker for the current speaker mode.
    ///
    /// This function allows the user to specify the position of their speaker
    /// to account for non standard setups.  
    /// It also allows the user to disable speakers from 3D consideration in a
    /// game.
    pub fn set_speaker_position(&self, speaker: Speaker, position: SpeakerPosition) -> Result {
        let SpeakerPosition { x, y, active } = position;
        ffi!(FMOD_System_SetSpeakerPosition(
            self.as_raw(),
            speaker.into_raw(),
            x,
            y,
            if active { 1 } else { 0 },
        ))?;
        Ok(())
    }

    /// Retrieves the position of the specified speaker for the current speaker
    /// mode.
    pub fn get_speaker_position(&self, speaker: Speaker) -> Result<SpeakerPosition> {
        let mut speaker_position = SpeakerPosition::default();
        let mut active = 0;
        ffi!(FMOD_System_GetSpeakerPosition(
            self.as_raw(),
            speaker.into_raw(),
            &mut speaker_position.x,
            &mut speaker_position.y,
            &mut active,
        ))?;
        speaker_position.active = active != 1;
        Ok(speaker_position)
    }

    /// Sets the global doppler scale, distance factor and log rolloff scale for
    /// all 3D sound in FMOD.
    ///
    /// See [Settings3d] for a description of what specificially this changes.
    pub fn set_3d_settings(&self, settings: Settings3d) -> Result {
        let Settings3d {
            doppler_scale,
            distance_factor,
            rolloff_scale,
        } = settings;
        ffi!(FMOD_System_Set3DSettings(
            self.as_raw(),
            doppler_scale,
            distance_factor,
            rolloff_scale,
        ))?;
        Ok(())
    }

    /// Retrieves the global doppler scale, distance factor and rolloff scale for all 3D sounds.
    pub fn get_3d_settings(&self) -> Result<Settings3d> {
        let mut settings = Settings3d::default();
        ffi!(FMOD_System_Get3DSettings(
            self.as_raw(),
            &mut settings.doppler_scale,
            &mut settings.distance_factor,
            &mut settings.rolloff_scale,
        ))?;
        Ok(settings)
    }

    /// Sets the number of 3D 'listeners' in the 3D sound scene.
    ///
    /// This function is useful mainly for split-screen game purposes.
    ///
    /// If the number of listeners is set to more than 1, then panning and
    /// doppler are turned off. All sound effects will be mono. FMOD uses a
    /// 'closest sound to the listener' method to determine what should be heard
    /// in this case.
    ///
    /// Users of the Studio API should call [studio::System::set_num_listeners]
    /// instead of this function.
    pub fn set_3d_num_listeners(&self, num_listeners: i32) -> Result {
        ffi!(FMOD_System_Set3DNumListeners(self.as_raw(), num_listeners))?;
        Ok(())
    }

    /// Retrieves the number of 3D listeners.
    ///
    /// Users of the Studio API should call [studio::System::get_num_listeners]
    /// instead of this function.
    pub fn get_3d_num_listeners(&self) -> Result<i32> {
        let mut num_listeners = 0;
        ffi!(FMOD_System_Get3DNumListeners(
            self.as_raw(),
            &mut num_listeners,
        ))?;
        Ok(num_listeners)
    }

    /// Sets a callback to allow custom calculation of distance attenuation.
    ///
    /// This function overrides [Mode::InverseRolloff3d],
    /// [Mode::LinearRolloff3d], [Mode::LinearSquareRolloff3d],
    /// [Mode::InverseTaperedRolloff3d], and [Mode::CustomRolloff3d].
    pub fn set_3d_rolloff_callback(&self, callback: Option<Rolloff3dCallback>) -> Result {
        ffi!(FMOD_System_Set3DRolloffCallback(
            self.as_raw(),
            mem::transmute(callback),
        ))?;
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

fmod_struct! {
    /// Advanced configuration settings.
    ///
    /// Structure to allow configuration of lesser used system level settings.
    /// These tweaks generally allow the user to set resource limits and
    /// customize settings to better fit their application.
    ///
    /// 0 means to not change the setting (and this is provided by `default()`),
    /// so setting only a few members is a common use pattern.
    ///
    /// Specifying one of the codec maximums will help determine the maximum CPU
    /// usage of playing [Mode::CreateCompressedSample] Sounds of that type as well
    /// as the memory requirements. Memory will be allocated for 'up front' (during
    /// [System::init]) if these values are specified as non zero. If any are zero,
    /// it allocates memory for the codec whenever a file of the type in question is
    /// loaded. So if `max_mpeg_codecs` is 0 for example, it will allocate memory
    /// for the MPEG codecs the first time an MP3 is loaded or an MP3 based .FSB
    /// file is loaded.
    ///
    /// Setting `dsp_buffer_pool_size` will pre-allocate memory for the FMOD DSP
    /// network. See [DSP architecture guide]. By default 8 buffers are created up
    /// front. A large network might require more if the aim is to avoid real-time
    /// allocations from the FMOD mixer thread.
    ///
    /// [DSP architecture guide]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-dsp-architecture.html
    pub struct AdvancedSettings = FMOD_ADVANCEDSETTINGS {
        /// Size of this structure. Must be set to `size_of::<Self>()`.
        #[default(mem::size_of::<Self>() as i32)]
        size: i32,
        /// Maximum MPEG Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_mpeg_codecs: i32,
        /// Maximum IMA-ADPCM Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_adpcm_codecs: i32,
        /// Maximum XMA Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_xma_codecs: i32,
        /// Maximum Vorbis Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_vorbix_codecs: i32,
        /// Maximum AT9 Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_at9_codecs: i32,
        /// Maximum FADPCM Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_fadpcm_codecs: i32,
        /// Deprecated.
        max_pcm_codecs: i32,
        /// Number of elements in `asio_speaker_list` on input, number of elements
        /// in `asio_channel_list` on output.
        /// <dl>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        asio_num_channels: i32,
        /// Read only list of strings representing ASIO channel names, count is
        /// defined by `asio_num_channels`. Only valid after [System::init].
        #[default(ptr::null_mut())]
        asio_channel_list: *mut *mut c_char,
        /// List of speakers that represent each ASIO channel used for remapping,
        /// count is defined by `asio_num_channels`. Use [Speaker::None] to indicate
        /// no output for a given speaker.
        #[default(ptr::null_mut())]
        asio_speaker_list: *mut FMOD_SPEAKER,
        /// For use with [InitFlags::Vol0BecomesVirtual], [Channel]s with audibility
        /// below this will become virtual. See the [Virtual Voices] guide for more
        /// information.
        ///
        /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
        /// <dl>
        /// <dt>Units</dt><dd>Linear</dd>
        /// <dt>Default</dt><dd>0</dd>
        /// </dl>
        pub vol_0_virtual_vol: f32,
        /// For use with Streams, the default size of the double buffer.
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>400</dd>
        /// <dt>Range</dt><dd>[0, 30000]</dd>
        /// </dl>
        pub default_decode_buffer_size: u32,
        /// For use with [InitFlags::ProfileEnable], specify the port to listen on
        /// for connections by FMOD Studio or FMOD Profiler.
        /// <dl>
        /// <dt>Default</dt><dd>9264</dd>
        /// </dl>
        pub profile_port: u16,
        /// For use with [Geometry], the maximum time it takes for a [Channel] to
        /// fade to the new volume level when its occlusion changes.
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>500</dd>
        /// </dl>
        pub geometry_max_fade_time: u32,
        /// For use with [InitFlags::ChannelDistanceFilter], the default center
        /// frequency for the distance filtering effect.
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>1500</dd>
        /// <dt>Range</dt><dd>[10, 22050]</dd>
        /// </dl>
        pub distance_filter_center_freq: f32,
        /// For use with [Reverb3D], selects which global reverb instance to use.
        /// <dl>
        /// <dt>Range</dt><dd>[0, MAX_INSTANCES]</dd>
        /// </dl>
        pub reverb_3d_instance: i32,
        /// Number of intermediate mixing buffers in the 'DSP buffer pool'. Each
        /// buffer in bytes will be `buffer_length` (See [System::get_dsp_buffer_size])
        /// × `size_of::<f32>()` × output mode speaker count (See [SpeakerMode]).
        /// ie 7.1 @ 1024 DSP block size = 1024 × 4 × 8 = 32KB.
        /// <dl>
        /// <dt>Default</dt><dd>8</dd>
        /// </dl>
        pub dsp_buffer_pool_size: i32,
        /// Resampling method used by [Channel]s.
        pub resampler_method: DspResampler,
        /// Seed value to initialize the internal random number generator.
        pub random_seed: u32,
        /// Maximum number of CPU threads to use for [DspType::Convolutionreverb]
        /// effect. 1 = effect is entirely processed inside the [ThreadType::Mixer]
        /// thread. 2 and 3 offloads different parts of the convolution processing
        /// into different threads ([ThreadType::Convolution1] and
        /// [ThreadType::Convolution2] to increase throughput.
        /// <dl>
        /// <dt>Default</dt><dd>3</dd>
        /// <dt>Range</dt><dd>[0, 3]</dd>
        /// </dl>
        pub max_convolution_threads: i32,
        /// Maximum Opus Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_opus_codecs: i32,
    }
}

impl AdvancedSettings {
    /// ASIO channel names. Only valid after [System::init].
    pub fn asio_channel_list(&self) -> Option<impl Iterator<Item = Cow<'_, str>>> {
        if self.asio_channel_list.is_null() {
            None
        } else {
            Some(
                unsafe {
                    slice::from_raw_parts(self.asio_channel_list, ix!(self.asio_num_channels))
                }
                .iter()
                .copied()
                .map(|ptr| unsafe { CStr::from_ptr(ptr) })
                .map(CStr::to_bytes)
                .map(String::from_utf8_lossy),
            )
        }
    }

    /// List of speakers that represent each ASIO channel used for remapping.
    pub fn asio_speaker_list(&self) -> Option<&[Speaker]> {
        if self.asio_speaker_list.is_null() {
            None
        } else {
            Some(unsafe {
                slice::from_raw_parts(self.asio_speaker_list.cast(), ix!(self.asio_num_channels))
            })
        }
    }
}

/// Callback to allow custom calculation of distance attenuation.
pub type Rolloff3dCallback = extern "system" fn(channel: &Channel, distance: f32) -> f32;

// -------------------------------------------------------------------------------------------------

enum_struct! {
    /// List of interpolation types used for resampling.
    ///
    /// Use [System::set_advanced_settings] and [AdvancedSettings::resampler_method] to configure the resampling quality you require for sample rate conversion during sound playback.
    pub enum DspResampler: FMOD_DSP_RESAMPLER {
        #[default]
        /// Default interpolation method, same as [DspResampler::Linear].
        Default  = FMOD_DSP_RESAMPLER_DEFAULT,
        /// No interpolation. High frequency aliasing hiss will be audible depending on the sample rate of the sound.
        NoInterp = FMOD_DSP_RESAMPLER_NOINTERP,
        /// Linear interpolation (default method). Fast and good quality, causes very slight lowpass effect on low frequency sounds.
        Linear   = FMOD_DSP_RESAMPLER_LINEAR,
        /// Cubic interpolation. Slower than linear interpolation but better quality.
        Cubic    = FMOD_DSP_RESAMPLER_CUBIC,
        /// 5 point spline interpolation. Slowest resampling method but best quality.
        Spline   = FMOD_DSP_RESAMPLER_SPLINE,
    }
}
// -------------------------------------------------------------------------------------------------

/// Output format for the software mixer.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SoftwareFormat {
    /// Sample rate of the mixer.
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[8000, 192000]</dd>
    /// <dt>Units</dt><dd>Hertz</dd>
    /// <dt>Default</dt><dd>48000</dd>
    /// </dl>
    pub sample_rate: i32,
    /// Speaker setup of the mixer.
    pub speaker_mode: SpeakerMode,
    /// Number of speakers for [SpeakerMode::Raw].
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[0, MAX_CHANNEL_WIDTH]</dd>
    /// </dl>
    pub num_raw_speakers: i32,
}

/// The buffer size for the FMOD software mixing engine.
#[derive(Debug, SmartDefault, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DspBufferSize {
    /// The mixer engine block size. Use this to adjust mixer update
    /// granularity. See below for more information on buffer length vs latency.
    ///
    /// <dl>
    /// <dt>Units</dt><dd>Samples</dd>
    /// <dt>Default</dt><dd>1024</dd>
    /// </dl>
    #[default(1024)]
    pub buffer_length: u32,
    /// The mixer engine number of buffers used. Use this to adjust mixer
    /// latency. See [System::set_dsp_buffer_size] for more information on
    /// number of buffers vs latency.
    pub num_buffers: i32,
}

/// The position of a speaker for the current speaker mode.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct SpeakerPosition {
    /// 2D X position relative to the listener. -1 = left, 0 = middle,
    /// +1 = right.
    /// <dl>
    /// <dt>Range</dt><dd>[-1, 1]</dd>
    /// </dl>
    pub x: f32,
    /// 2D Y position relative to the listener. -1 = back, 0 = middle,
    /// +1 = front.
    /// <dl>
    /// <dt>Range</dt><dd>[-1, 1]</dd>
    /// </dl>
    pub y: f32,
    /// Active state of a speaker. true = included in 3D calculations,
    /// false = ignored.
    pub active: bool,
}

/// The global doppler scale, distance factor and log rolloff scale for all 3D
/// sound in FMOD.
#[derive(Debug, SmartDefault, Copy, Clone, PartialEq)]
pub struct Settings3d {
    /// A general scaling factor for how much the pitch varies due to doppler
    /// shifting in 3D sound. Doppler is the pitch bending effect when a sound
    /// comes towards the listener or moves away from it, much like the effect
    /// you hear when a train goes past you with its horn sounding. With
    /// `doppler_scale` you can exaggerate or diminish the effect. FMOD's
    /// effective speed of sound at a doppler factor of 1.0 is 340 m/s.
    #[default(1.0)]
    pub doppler_scale: f32,
    /// The FMOD 3D engine relative distance factor, compared to 1.0 meters.
    /// Another way to put it is that it equates to "how many units per meter
    /// does your engine have". For example, if you are using feet then "scale"
    /// would equal 3.28.  
    /// This only affects doppler. If you keep your min/max distance, custom
    /// rolloff curves and positions in scale relative to each other the volume
    /// rolloff will not change. If you set this, the min_distance of a sound
    /// will automatically set itself to this value when it is created in case
    /// the user forgets to set the min_distance to match the new
    /// distance_factor.
    #[default(1.0)]
    pub distance_factor: f32,
    /// The global attenuation rolloff factor. Volume for a sound will scale at
    /// min_distance / distance. Setting this value makes the sound drop off
    /// faster or slower. The higher the value, the faster volume will
    /// attenuate, and conversely the lower the value, the slower it will
    /// attenuate. For example, a rolloff factor of 1 will simulate the real
    /// world, where as a value of 2 will make sounds attenuate 2 times quicker.
    #[default(1.0)]
    pub rolloff_scale: f32,
}

// -------------------------------------------------------------------------------------------------

/// # File system setup.
impl System {
    /// Set file I/O to use the platform native method.
    ///
    /// `block_align` is the file buffering chunk size; specify -1 to keep the
    /// system default or previously set value. 0 = disable buffering.
    ///
    /// Setting `block_align` to 0 will disable file buffering and cause every
    /// read to invoke the relevant callback (not recommended), current default
    /// is tuned for memory usage vs performance. Be mindful of the I/O
    /// capabilities of the platform before increasing this default.
    pub fn set_file_system_default(&self, block_align: i32) -> Result {
        ffi!(FMOD_System_SetFileSystem(
            self.as_raw(),
            None,
            None,
            None,
            None,
            None,
            None,
            block_align,
        ))?;
        Ok(())
    }

    /// Set callbacks to implement all file I/O instead of using the platform
    /// native method.
    ///
    /// `block_align` is the file buffering chunk size; specify -1 to keep the
    /// system default or previously set value. 0 = disable buffering.
    ///
    /// Setting `block_align` to 0 will disable file buffering and cause every
    /// read to invoke the relevant callback (not recommended), current default
    /// is tuned for memory usage vs performance. Be mindful of the I/O
    /// capabilities of the platform before increasing this default.
    pub fn set_file_system_sync<FS: file::SyncFileSystem>(&self, block_align: i32) -> Result {
        ffi!(FMOD_System_SetFileSystem(
            self.as_raw(),
            Some(file::useropen::<FS>),
            Some(file::userclose::<FS>),
            Some(file::userread::<FS>),
            Some(file::userseek::<FS>),
            None,
            None,
            block_align,
        ))?;
        Ok(())
    }

    /// Set callbacks to implement all file I/O instead of using the platform
    /// native method.
    ///
    /// `block_align` is the file buffering chunk size; specify -1 to keep the
    /// system default or previously set value. 0 = disable buffering.
    ///
    /// Setting `block_align` to 0 will disable file buffering and cause every
    /// read to invoke the relevant callback (not recommended), current default
    /// is tuned for memory usage vs performance. Be mindful of the I/O
    /// capabilities of the platform before increasing this default.
    ///
    /// # Asynchrony notes
    ///
    /// - It is recommended to consult the 'async_io' example for reference
    /// implementation. There is also a tutorial on the subject,
    /// [Asynchronous I/O](https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-asynchronous-io.html).
    /// - [`AsyncFileSystem::read`] allows the user to return immediately before
    /// the data is ready. FMOD will either wait internally (see note below
    /// about thread safety), or continuously check in the streamer until data
    /// arrives. It is the user's responsibility to provide data in time in the
    /// stream case, or the stream may stutter. Data starvation can be detected
    /// with [Sound::get_open_state].
    /// - **Important:** If [`AsyncFileSystem::read`] is processed in the main
    /// thread, then it will hang the application, because FMOD will wait
    /// internally until data is ready, and the main thread will not be able to
    /// supply the data. For this reason the user's file access should normally
    /// be from a separate thread.
    /// - [AsyncFileSystem::cancel] must either service or prevent an async read
    /// issued previously via [AsyncFileSystem::read] before returning.
    pub fn set_file_system_async<FS: file::AsyncFileSystem>(&self, block_align: i32) -> Result {
        ffi!(FMOD_System_SetFileSystem(
            self.as_raw(),
            Some(file::useropen::<FS>),
            Some(file::userclose::<FS>),
            None,
            None,
            Some(file::userasyncread::<FS>),
            Some(file::userasynccancel::<FS>),
            block_align,
        ))?;
        Ok(())
    }

    /// 'Piggyback' on FMOD file reading routines to capture data as it's read.
    ///
    /// This allows users to capture data as FMOD reads it, which may be useful
    /// for extracting the raw data that FMOD reads for hard to support sources
    /// (for example internet streams).
    ///
    /// To detach, use [`detach_file_system`].
    ///
    /// Note: This function is not to replace FMOD's file system. For this
    /// functionality, see [System::set_file_system].
    pub fn attach_file_system<FS: file::ListenFileSystem>(&self) -> Result {
        ffi!(FMOD_System_AttachFileSystem(
            self.as_raw(),
            Some(file::useropen_listen::<FS>),
            Some(file::userclose_listen::<FS>),
            Some(file::userread_listen::<FS>),
            Some(file::userseek_listen::<FS>),
        ))?;
        Ok(())
    }

    /// Detach a previously [attached](Self::attach_file_system) file system
    /// listener.
    pub fn detach_file_system(&self) -> Result {
        ffi!(FMOD_System_AttachFileSystem(
            self.as_raw(),
            None,
            None,
            None,
            None,
        ))?;
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

/// # Plugin support.
impl System {
    /// Specify a base search path for plugins so they can be placed somewhere
    /// else than the directory of the main executable.
    pub fn set_plugin_path(&self, path: &CStr8) -> Result {
        ffi!(FMOD_System_SetPluginPath(self.as_raw(), path.as_ptr() as _))?;
        Ok(())
    }

    // TODO: figure out plugin API
    /*
        /// Loads an FMOD (DSP, Output or Codec) plugin from file.
        ///
        /// Once loaded, DSP plugins can be used via [System::create_dsp_by_plugin],
        /// output plugins can be use via [System::set_output_by_plugin], and codec
        /// plugins will be used automatically.
        ///
        /// When opening a file each codec tests whether it can support the file
        /// format in `priority` order where 0 represents most important and higher
        /// numbers represent less importance.
        ///
        /// The format of the plugin is dependant on the operating system.
        pub fn load_plugin(&self, filename: &CStr8, priority: u32) -> Result<PluginHandle> {
            let mut handle = PluginHandle::default();
            ffi!(FMOD_System_LoadPlugin(
                self.as_raw(),
                filename.as_ptr() as _,
                handle.as_raw_mut(),
                priority,
            ))?;
            Ok(handle)
        }

        /// Unloads an FMOD (DSP, Output or Codec) plugin.
        pub fn unload_plugin(&self, handle: PluginHandle) -> Result {
            ffi!(FMOD_System_UnloadPlugin(self.as_raw(), handle.into_raw()))?;
            Ok(())
        }

        /// Retrieves the number of nested plugins from the selected plugin.
        ///
        /// Most plugins contain a single definition, in which case the count is 1,
        /// however some have a list of definitions. This function returns the
        /// number of plugins that have been defined.
        ///
        /// See the [DSP Plug-in API guide](https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-dsp-plugin-api.html#multiple-plugins-within-one-file)
        /// for more information.
        pub fn get_num_nested_plugins(&self, handle: PluginHandle) -> Result<i32> {
            let mut count = 0;
            ffi!(FMOD_System_GetNumNestedPlugins(
                self.as_raw(),
                handle.into_raw(),
                &mut count,
            ))?;
            Ok(count)
        }

        /// Retrieves the handle of a nested plugin.
        ///
        /// This function is used to iterate handles for plugins that have a list of
        /// definitions.
        ///
        /// Most plugins contain a single definition. If this is the case, only
        /// index 0 is valid, and the returned handle is the same as the handle
        /// passed in.
        pub fn get_nested_plugin(&self, handle: PluginHandle, index: i32) -> Result<PluginHandle> {
            let mut nested_handle = PluginHandle::default();
            ffi!(FMOD_System_GetNestedPlugin(
                self.as_raw(),
                handle.into_raw(),
                index,
                nested_handle.as_raw_mut(),
            ))?;
            Ok(nested_handle)
        }

        /// Retrieves the number of loaded plugins.
        pub fn get_num_plugins(&self, plugin_type: PluginType) -> Result<i32> {
            let mut num_plugins = 0;
            ffi!(FMOD_System_GetNumPlugins(
                self.as_raw(),
                plugin_type.into_raw(),
                &mut num_plugins,
            ))?;
            Ok(num_plugins)
        }

        /// Retrieves the handle of a plugin based on its type and relative index.
        ///
        /// All plugins whether built in or loaded can be enumerated using this and
        /// [System::get_num_plugins].
        pub fn get_plugin_handle(&self, plugin_type: PluginType, index: i32) -> Result<PluginHandle> {
            let mut handle = PluginHandle::default();
            ffi!(FMOD_System_GetPluginHandle(
                self.as_raw(),
                plugin_type.into_raw(),
                index,
                handle.as_raw_mut(),
            ))?;
            Ok(handle)
        }

        // NB: we split get_plugin_info/name into separate calls for two reasons:
        // getting everything *but* the name is cheap, and the name has extra retry
        // requirements to validate non-truncation and UTF-8. This does, however,
        // mean that getting all of the plugin info requires an extra FFI call.

        /// Retrieves information for the selected plugin.
        pub fn get_plugin_info(&self, handle: PluginHandle) -> Result<PluginInfo> {
            let mut kind = PluginType::zeroed();
            let mut version = 0;
            ffi!(FMOD_System_GetPluginInfo(
                self.as_raw(),
                handle.raw,
                kind.as_raw_mut(),
                ptr::null_mut(),
                0,
                &mut version,
            ))?;
            Ok(PluginInfo { kind, version })
        }

        /// Retrieves name for the selected plugin.
        pub fn get_plugin_name(&self, handle: PluginHandle, name: &mut String) -> Result {
            unsafe {
                fmod_get_string(name, |buf| {
                    ffi!(FMOD_System_GetPluginInfo(
                        self.as_raw(),
                        handle.raw,
                        ptr::null_mut(),
                        buf.as_mut_ptr().cast(),
                        buf.len() as _,
                        ptr::null_mut(),
                    ))
                })
            }
        }

        /// Selects an output type given a plugin handle.
        ///
        /// (Windows Only) This function can be called after FMOD is already
        /// initialized. You can use it to change the output mode at runtime. If
        /// [raw::FMOD_SYSTEM_CALLBACK_DEVICELISTCHANGED] is specified use the
        /// set_output call to change to [OutputType::NoSound] if no more sound card
        /// drivers exist.
        pub fn set_output_by_plugin(&self, handle: PluginHandle) -> Result {
            ffi!(FMOD_System_SetOutputByPlugin(
                self.as_raw(),
                handle.into_raw(),
            ))?;
            Ok(())
        }

        /// Retrieves the plugin handle for the currently selected output type.
        pub fn get_output_by_plugin(&self) -> Result<PluginHandle> {
            let mut handle = PluginHandle::default();
            ffi!(FMOD_System_GetOutputByPlugin(
                self.as_raw(),
                handle.as_raw_mut(),
            ))?;
            Ok(handle)
        }

        /// Create a DSP object given a plugin handle.
        ///
        /// A DSP object is a module that can be inserted into the mixing graph to
        /// allow sound filtering or sound generation. See the [DSP architecture guide](https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-dsp-architecture.html)
        /// for more information.
        ///
        /// A handle can come from a newly loaded plugin with [System::load_plugin]
        /// or an existing plugin with [System::get_plugin_handle].
        ///
        /// DSPs must be attached to the DSP graph before they become active, either
        /// via [ChannelControl::add_dsp] or [Dsp::add_input].
        pub fn create_dsp_by_plugin(&self, handle: PluginHandle) -> Result<Handle<'_, Dsp>> {
            let mut dsp = ptr::null_mut();
            ffi!(FMOD_System_CreateDSPByPlugin(
                self.as_raw(),
                handle.into_raw(),
                &mut dsp,
            ))?;
            Ok(unsafe { Handle::new(dsp) })
        }

        // this functionality needs to wait for custom plugin author binding work.
        // pub fn get_dsp_info_by_plugin(&self, handle: PluginHandle) -> Result<DspDescription> {
        // pub fn register_codec(priority: u32, desc: CodecPluginDescription) -> Result<PluginHandle>;
        // pub fn register_dsp(description: DspDescription) -> Result<PluginHandle>;
        // pub fn register_output(description: OutputDescription) -> Result<PluginHandle>;
    */
}

// -------------------------------------------------------------------------------------------------

enum_struct! {
    /// Types of plugin used to extend functionality.
    pub enum PluginType: FMOD_PLUGINTYPE {
        /// Audio output interface plugin represented with [OutputDescription].
        Output = FMOD_PLUGINTYPE_OUTPUT,
        /// File format codec plugin represented with [CodecDescription].
        Codec  = FMOD_PLUGINTYPE_CODEC,
        /// DSP unit plugin represented with [DspDescription].
        Dsp    = FMOD_PLUGINTYPE_DSP,
    }
}

// -------------------------------------------------------------------------------------------------

/// Information about a selected plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// Plugin type.
    pub kind: PluginType,
    /// Version number of the plugin.
    pub version: u32,
}

// -------------------------------------------------------------------------------------------------

/// # Network configuration.
impl System {
    /// Set a proxy server to use for all subsequent internet connections.
    ///
    /// Specify the proxy in `host:port` format e.g. `www.fmod.com:8888`
    /// (defaults to port 80 if no port is specified).
    ///
    /// Basic authentication is supported using `user:password@host:port` format
    /// e.g. `bob:sekrit123@www.fmod.com:8888`.
    pub fn set_network_proxy(&self, proxy: &CStr8) -> Result {
        ffi!(FMOD_System_SetNetworkProxy(
            self.as_raw(),
            proxy.as_ptr() as _,
        ))?;
        Ok(())
    }

    /// Retrieves the URL of the proxy server used in internet streaming.
    pub fn get_network_proxy(&self, proxy: &mut String) -> Result {
        unsafe {
            fmod_get_string(proxy, |buf| {
                ffi!(FMOD_System_GetNetworkProxy(
                    self.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                ))
            })
        }
    }

    /// Set the timeout for network streams.
    pub fn set_network_timeout(&self, timeout: Duration) -> Result {
        let timeout = timeout.as_millis() as _;
        ffi!(FMOD_System_SetNetworkTimeout(self.as_raw(), timeout))?;
        Ok(())
    }

    /// Retrieve the timeout value for network streams.
    pub fn get_network_timeout(&self) -> Result<Duration> {
        let mut timeout = 0;
        ffi!(FMOD_System_GetNetworkTimeout(self.as_raw(), &mut timeout))?;
        Ok(Duration::from_millis(timeout as _))
    }
}

// -------------------------------------------------------------------------------------------------

/// # Information.
impl System {
    /// Retrieves the FMOD version number.
    ///
    /// Compare against `fmod::VERSION` to make sure header and runtime library
    /// versions match.
    pub fn get_version(&self) -> Result<Version> {
        let mut version = 0;
        ffi!(FMOD_System_GetVersion(self.as_raw(), &mut version))?;
        Ok(Version::from_raw(version))
    }

    /// Retrieves an output type specific internal native interface.
    ///
    /// Reinterpret the returned handle based on the selected output type, not
    /// all types return something.
    ///
    /// - [`OutputType::WavWriter`]: Pointer to stdio `FILE` is returned.
    /// - [`OutputType::WavWriterNrt`]: Pointer to stdio `FILE` is returned.
    /// - [`OutputType::Wasapi`]: Pointer to type `IAudioClient` is returned.
    /// - [`OutputType::Alsa`]: Pointer to type `snd_pcm_t` is returned.
    /// - [`OutputType::CoreAudio`]: Handle of type `AudioUnit` is returned.
    /// - [`OutputType::AudioOut`]: Pointer to type `i32` is returned. Handle returned from `sceAudioOutOpen`.
    pub fn get_output_handle(&self) -> Result<*mut ()> {
        let mut output = ptr::null_mut();
        ffi!(FMOD_System_GetOutputHandle(self.as_raw(), &mut output))?;
        Ok(output.cast())
    }

    /// Retrieves the number of currently playing Channels.
    ///
    /// For differences between real and virtual voices see the Virtual Voices
    /// guide for more information.
    pub fn get_channels_playing(&self) -> Result<ChannelUsage> {
        let mut channels = 0;
        let mut real_channels = 0;
        ffi!(FMOD_System_GetChannelsPlaying(
            self.as_raw(),
            &mut channels,
            &mut real_channels,
        ))?;
        Ok(ChannelUsage {
            all: channels,
            real: real_channels,
        })
    }

    /// Retrieves the amount of CPU used for different parts of the Core engine.
    ///
    /// For readability, the percentage values are smoothed to provide a more
    /// stable output.
    pub fn get_cpu_usage(&self) -> Result<CpuUsage> {
        let mut usage = CpuUsage::default();
        ffi!(FMOD_System_GetCPUUsage(self.as_raw(), usage.as_raw_mut()))?;
        Ok(usage)
    }

    /// Retrieves information about file reads.
    pub fn get_file_usage(&self) -> Result<FileUsage> {
        let mut sample_bytes_read = 0;
        let mut stream_bytes_read = 0;
        let mut other_bytes_read = 0;
        ffi!(FMOD_System_GetFileUsage(
            self.as_raw(),
            &mut sample_bytes_read,
            &mut stream_bytes_read,
            &mut other_bytes_read,
        ))?;
        Ok(FileUsage {
            sample_bytes_read,
            stream_bytes_read,
            other_bytes_read,
        })
    }

    /// TODO: figure out mix matrix API
    // get_default_mix_matrix

    /// Retrieves the channel count for a given speaker mode.
    pub fn get_speaker_mode_channels(&self, mode: SpeakerMode) -> Result<usize> {
        let mut channels = 0;
        ffi!(FMOD_System_GetSpeakerModeChannels(
            self.as_raw(),
            mode.into_raw(),
            &mut channels,
        ))?;
        Ok(channels as _)
    }
}

// -------------------------------------------------------------------------------------------------

/// A number of playing channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelUsage {
    /// Number of playing [Channel]s (both real and virtual).
    pub all: i32,
    /// Number of playing real (non-virtual) [Channel]s.
    pub real: i32,
}

/// Running total information about file reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileUsage {
    /// Total bytes read from file for loading sample data.
    pub sample_bytes_read: i64,
    /// Total bytes read from file for streaming sounds.
    pub stream_bytes_read: i64,
    /// Total bytes read for non-audio data such as FMOD Studio banks.
    pub other_bytes_read: i64,
}

// -------------------------------------------------------------------------------------------------

/// # Creation and retrieval.
impl System {
    /// Loads a sound into memory, opens it for streaming or sets it up for
    /// callback based sounds.
    ///
    /// [Mode::CreateSample] will try to load and decompress the whole sound
    /// into memory, use [Mode::CreateStream] to open it as a stream and have
    /// it play back in realtime from disk or another medium.
    /// [Mode::CreateCompressedSample] can also be used for certain formats to
    /// play the sound directly in its compressed format from the mixer.
    ///
    /// - To open a file or URL as a stream, so that it decompresses / reads at
    ///   runtime, instead of loading / decompressing into memory all at the
    ///   time of this call, use the [Mode::CreateStream] flag.
    /// - To open a file or URL as a compressed sound effect that is not
    ///   streamed and is not decompressed into memory at load time, use
    ///   [Mode::CreateCompressedSample]. This is supported with MPEG (mp2/mp3),
    ///   ADPCM/FADPCM, XMA, AT9 and FSB Vorbis files only. This is useful for
    ///   those who want realtime compressed soundeffects, but not the overhead
    ///   of disk access.
    /// - To open a sound as 2D, so that it is not affected by 3D processing,
    ///   use the [Mode::D2] flag. 3D sound commands will be ignored on these
    ///   types of sounds.
    /// - To open a sound as 3D, so that it is treated as a 3D sound, use the
    ///   [Mode::D3] flag.
    ///
    /// Note that [Mode::OpenRaw], [Mode::OpenMemory], [Mode::OpenMemoryPoint],
    /// and [Mode::OpenUser] will not work here, as more information is needed.
    /// Use [`create_sound_ex`](Self::create_sound_ex) instead.
    ///
    /// Use [Mode::NonBlocking] to have the sound open or load in the
    /// background. You can use [Sound::get_open_state] to determine if it has
    /// finished loading / opening or not. While it is loading (not ready),
    /// sound functions are not accessible for that sound.
    ///
    /// To account for slow media that might cause buffer underrun (skipping /
    /// stuttering / repeating blocks of audio) with sounds created with
    /// [Mode::CreateStream], use [System::set_stream_buffer_size] to increase
    /// read ahead.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Use of Mode::NonBlocking is currently not supported for Wasm.
    /// </span></div></div>
    pub fn create_sound(&self, name: &CStr8, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            whoops!("System::create_sound called with extended mode {mode:?}; use create_sound_ex instead");
            yeet!(Error::InvalidParam);
        }

        let mode = Mode::into_raw(mode);
        let exinfo = ptr::null_mut();
        let mut sound = ptr::null_mut();
        ffi!(FMOD_System_CreateSound(
            self.as_raw(),
            name.as_ptr() as _,
            mode,
            exinfo,
            &mut sound,
        ))?;
        Ok(unsafe { Handle::new(sound) })
    }

    /// Loads a sound into memory, opens it for streaming or sets it up for
    /// callback based sounds.
    ///
    /// Unlike [`create_sound`](Self::create_sound), this function allows you to
    /// specify additional configuration using the [`CreateSoundEx`] type.
    ///
    /// Note that [Mode::OpenRaw], [Mode::OpenMemory], [Mode::OpenMemoryPoint],
    /// and [Mode::OpenUser] will not work without required additional
    /// information provided in the [`CreateSoundEx`] type.
    ///
    /// Use [Mode::NonBlocking] to have the sound open or load in the
    /// background. You can use [Sound::get_open_state] to determine if it has
    /// finished loading / opening or not. While it is loading (not ready),
    /// sound functions are not accessible for that sound. Do not free memory
    /// provided with [Mode::OpenMemory] if the sound is not in a ready state,
    /// as it will most likely lead to UB and a crash.
    ///
    /// Specifying [`Mode::OpenMemoryPoint`] will POINT to your memory rather
    /// allocating its own sound buffers and duplicating it internally, this
    /// means you cannot free the memory while FMOD is using it, until after
    /// [`Sound::release`] is called.
    ///
    /// With [`Mode::OpenMemoryPoint`], only PCM formats and compressed formats
    /// using [`Mode::CreateCompressedSample`] are supported.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Use of Mode::NonBlocking is currently not supported for Wasm.
    /// </span></div></div>
    pub fn create_sound_ex(&self) -> CreateSoundEx<'_> {
        CreateSoundEx::new()
    }

    /// Opens a sound for streaming.
    ///
    /// This is a convenience function for [`System::create_sound`] with the
    /// [`Mode::CreateStream`] flag added.
    ///
    /// A stream only has one decode buffer and file handle, and therefore can
    /// only be played once. It cannot play multiple times at once because it
    /// cannot share a stream buffer if the stream is playing at different
    /// positions. Open multiple streams to have them play concurrently.
    ///
    /// If you need access to the extended options, use
    /// [`System::create_sound_ex`] instead and set [`Mode::CreateStream`].
    pub fn create_stream(&self, name: &CStr8, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            whoops!("System::create_stream called with extended mode {mode:?}; use create_sound_ex instead");
            yeet!(Error::InvalidParam);
        }

        let mode = Mode::into_raw(mode);
        let exinfo = ptr::null_mut();
        let mut sound = ptr::null_mut();
        ffi!(FMOD_System_CreateStream(
            self.as_raw(),
            name.as_ptr() as _,
            mode,
            exinfo,
            &mut sound,
        ))?;
        Ok(unsafe { Handle::new(sound) })
    }

    // TODO: pub fn create_dsp

    /// Create a DSP object given a built in type index.
    ///
    /// A DSP object is a module that can be inserted into the mixing graph to
    /// allow sound filtering or sound generation. See the
    /// [DSP architecture guide] for more information.
    ///
    /// [DSP architecture guide]: file:///C:/Program%20Files%20(x86)/FMOD%20SoundSystem/FMOD%20Studio%20API%20Windows/doc/FMOD%20API%20User%20Manual/white-papers-dsp-architecture.html
    ///
    /// DSPs must be attached to the DSP graph before they become active, either
    /// via [ChannelControl::add_dsp] or [Dsp::add_input].
    ///
    /// Using [DspType::VstPlugin] or [DspType::WinampPlugin] will return the
    /// first loaded plugin of this type. To access other plugins of these
    /// types, use [System::create_dsp_by_plugin] instead.
    pub fn create_dsp_by_type(&self, kind: DspType) -> Result<Handle<'_, Dsp>> {
        let mut dsp = ptr::null_mut();
        ffi!(FMOD_System_CreateDSPByType(
            self.as_raw(),
            kind.into_raw(),
            &mut dsp,
        ))?;
        Ok(unsafe { Handle::new(dsp) })
    }

    /// Create a ChannelGroup object.
    ///
    /// [ChannelGroup]s can be used to assign / group [Channel]s, for things
    /// such as volume scaling. [ChannelGroup]s are also used for sub-mixing.
    /// Any [Channel]s that are assigned to a [ChannelGroup] get submixed into
    /// that [ChannelGroup]'s 'tail' [Dsp]. See [ChannelControlDspIndex::Tail].
    ///
    /// If a [ChannelGroup] has an effect added to it, the effect is processed
    /// post-mix from the [Channel]s and [ChannelGroup]s below it in the mix
    /// hierarchy. See the [DSP architecture guide] for more information.
    ///
    /// [DSP architecture guide]: file:///C:/Program%20Files%20(x86)/FMOD%20SoundSystem/FMOD%20Studio%20API%20Windows/doc/FMOD%20API%20User%20Manual/white-papers-dsp-architecture.html
    ///
    /// All [ChannelGroup]s will initially output directly to the master
    /// [ChannelGroup] (See [System::get_master_channel_group]).[ChannelGroup]s
    /// can be re-parented this with [ChannelGroup::add_group].
    pub fn create_channel_group(&self, name: &CStr8) -> Result<Handle<'_, ChannelGroup>> {
        let mut channel_group = ptr::null_mut();
        ffi!(FMOD_System_CreateChannelGroup(
            self.as_raw(),
            name.as_ptr() as _,
            &mut channel_group,
        ))?;
        Ok(unsafe { Handle::new(channel_group) })
    }

    /// Creates a SoundGroup object.
    ///
    /// A [SoundGroup] is a way to address multiple [Sound]s at once with group
    /// level commands, such as:
    /// - Attributes of [Sound]s that are playing or about to be played, such as
    ///   volume. See [SoundGroup::set_volume].
    /// - Control of playback, such as stopping [Sound]s. See
    ///   [SoundGroup::stop].
    /// - Playback behavior such as 'max audible', to limit playback of certain
    ///   types of Sounds. See [SoundGroup::set_max_audible].
    pub fn create_sound_group(&self, name: &CStr8) -> Result<Handle<'_, SoundGroup>> {
        let mut sound_group = ptr::null_mut();
        ffi!(FMOD_System_CreateSoundGroup(
            self.as_raw(),
            name.as_ptr() as _,
            &mut sound_group,
        ))?;
        Ok(unsafe { Handle::new(sound_group) })
    }

    /// Creates a 'virtual reverb' object. This object reacts to 3D location and
    /// morphs the reverb environment based on how close it is to the reverb
    /// object's center.
    ///
    /// Multiple reverb objects can be created to achieve a multi-reverb
    /// environment. 1 Physical reverb object is used for all 3D reverb objects
    /// (slot 0 by default).
    ///
    /// The 3D reverb object is a sphere having 3D attributes (position, minimum
    /// distance, maximum distance) and reverb properties.
    ///
    /// The properties and 3D attributes of all reverb objects collectively
    /// determine, along with the listener's position, the settings of and
    /// input gains into a single 3D reverb [Dsp].
    ///
    /// When the listener is within the sphere of effect of one or more 3D
    /// reverbs, the listener's 3D reverb properties are a weighted combination
    /// of such 3D reverbs.
    ///
    /// When the listener is outside all of the reverbs, no reverb is applied.
    ///
    /// [System::set_reverb_properties] can be used to create an alternative
    /// reverb that can be used for 2D and background global reverb.
    ///
    /// To avoid this reverb interfering with the reverb slot used by the 3D
    /// reverb, 2D reverb should use a different slot id with
    /// [System::set_reverb_properties], otherwise
    /// [AdvancedSettings::reverb_3d_instance] can also be used to place 3D
    /// reverb on a different physical reverb slot.
    ///
    /// Use [ChannelControl::set_reverb_properties] to turn off reverb for 2D
    /// sounds (ie set wet = 0).
    ///
    /// Creating multiple reverb objects does not impact performance. These are
    /// 'virtual reverbs'. There will still be only one physical reverb [Dsp]
    /// running that just morphs between the different virtual reverbs.
    ///
    /// Note about physical reverb [Dsp] unit allocation. To remove the [Dsp]
    /// unit and the associated CPU cost, first make sure all 3D reverb objects
    /// are released. Then call [System::set_reverb_properties] with the 3D
    /// reverb's slot ID (default is 0) with a property point of 0 or NULL, to
    /// signal that the physical reverb instance should be deleted.
    ///
    /// If a 3D reverb is still present, and [System::set_reverb_properties]
    /// function is called to free the physical reverb, the 3D reverb system
    /// will immediately recreate it upon the next [System::update] call.
    ///
    /// Note that the 3D reverb system will not affect Studio events unless it
    /// is explicitly enabled by calling
    /// [studio::EventInstance::set_reverb_level] on each event instance.
    pub fn create_reverb_3d(&self) -> Result<Handle<'_, Reverb3d>> {
        let mut reverb = ptr::null_mut();
        ffi!(FMOD_System_CreateReverb3D(self.as_raw(), &mut reverb))?;
        Ok(unsafe { Handle::new(reverb) })
    }

    /// Plays a Sound on a Channel.
    ///
    /// When a sound is played, it will use the sound's default frequency and
    /// priority. See [Sound::set_defaults].
    ///
    /// A sound defined as [Mode::D3] will by default play at the 3D position of
    /// the listener. To set the 3D position of the [Channel] before the sound
    /// is audible, start the [Channel] paused by setting the `paused` parameter
    /// to true, and call [ChannelControl::set_3D_attributes].
    ///
    /// Specifying a `channel_group` as part of `play_sound` is more efficient
    /// than using [Channel::set_channel_group] after play_sound, and could
    /// avoid audible glitches if the play_sound is not in a paused state.
    ///
    /// [Channel]s are reference counted to handle dead or stolen [Channel]
    /// handles. See the white paper on [Channel handles] for more information.
    ///
    /// Playing more [Sound]s than physical [Channel]s allow is handled with
    /// virtual voices. See the white paper on [Virtual Voices] for more
    /// information.
    ///
    /// [Channel handles]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-handle-system.html#core-api-channels
    /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
    pub fn play_sound(
        &self,
        sound: &Sound,
        channel_group: Option<&ChannelGroup>,
        paused: bool,
    ) -> Result<&Channel> {
        let sound = Sound::as_raw(sound);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_PlaySound(
            self.as_raw(),
            sound,
            channelgroup,
            paused as _,
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    /// Plays a DSP along with any of its inputs on a Channel.
    ///
    /// Specifying a `channel_group` as part of play_dsp is more efficient
    /// than using `Channel::set_channel_group` after play_dsp, and could avoid
    /// audible glitches if the play_dsp is not in a paused state.
    ///
    /// [Channel]s are reference counted to handle dead or stolen [Channel]
    /// handles. See the white paper on [Channel handles] for more information.
    ///
    /// Playing more [Sound]s or [Dsp]s than physical [Channel]s allow is
    /// handled with virtual voices. See the white paper on [Virtual Voices]
    /// for more information.
    ///
    /// [Channel handles]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-handle-system.html#core-api-channels
    /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
    pub fn play_dsp(
        &self,
        dsp: &Dsp,
        channel_group: Option<&ChannelGroup>,
        paused: bool,
    ) -> Result<&Channel> {
        let dsp = Dsp::as_raw(dsp);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_PlayDSP(
            self.as_raw(),
            dsp,
            channelgroup,
            paused as _,
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    /// Retrieves a handle to a Channel by ID.
    ///
    /// This function is mainly for getting handles to existing (playing)
    /// [Channel]s and setting their attributes. The only way to 'create' an
    /// instance of a [Channel] for playback is to use [System::play_sound] or
    /// [System::play_dsp].
    pub fn get_channel(&self, channel_id: i32) -> Result<&Channel> {
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_GetChannel(
            self.as_raw(),
            channel_id,
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    // TODO: pub fn get_dsp_info_by_type

    /// Retrieves the master ChannelGroup that all sounds ultimately route to.
    ///
    /// This is the default [ChannelGroup] that [Channel]s play on, unless a
    /// different [ChannelGroup] is specified with [System::play_sound],
    /// [System::play_dsp], or [Channel::set_channel_group].  
    /// A master [ChannelGroup] can be used to do things like set the 'master
    /// volume' for all playing [Channel]s. See [ChannelControl::set_volume].
    pub fn get_master_channel_group(&self) -> Result<&ChannelGroup> {
        let mut channelgroup = ptr::null_mut();
        ffi!(FMOD_System_GetMasterChannelGroup(
            self.as_raw(),
            &mut channelgroup,
        ))?;
        Ok(unsafe { ChannelGroup::from_raw(channelgroup) })
    }

    /// Retrieves the default SoundGroup, where all sounds are placed when they
    /// are created.
    ///
    /// If [SoundGroup] is released, the [Sound]s will be put back into this
    /// [SoundGroup].
    pub fn get_master_sound_group(&self) -> Result<&SoundGroup> {
        let mut soundgroup = ptr::null_mut();
        ffi!(FMOD_System_GetMasterSoundGroup(
            self.as_raw(),
            &mut soundgroup,
        ))?;
        Ok(unsafe { SoundGroup::from_raw(soundgroup) })
    }
}

// -------------------------------------------------------------------------------------------------

/// # Runtime control.
impl System {
    /// Sets the position, velocity and orientation of the specified 3D sound
    /// listener.
    ///
    /// The `forward` and `up` vectors must be perpendicular and be of unit
    /// length (magnitude of each vector should be 1).
    ///
    /// Vectors must be provided in the correct [handedness].
    ///
    /// [handedness]: https://fmod.com/resources/documentation-api?version=2.02&page=glossary.html#handedness
    ///
    /// For velocity, remember to use units per **second**, and not units per
    /// frame. This is a common mistake and will make the doppler effect sound
    /// wrong if velocity is based on movement per frame rather than a fixed
    /// time period.  
    /// If velocity per frame is calculated, it can be converted to velocity per
    /// second by dividing it by the time taken between frames as a fraction of
    /// a second.  
    /// i.e.
    ///
    /// ```rust
    /// # let [position_currentframe, position_lastframe, time_taken_since_last_frame_in_seconds] = [1; 3];
    /// let velocity_units_per_second =
    ///     (position_currentframe - position_lastframe)
    ///         / time_taken_since_last_frame_in_seconds;
    /// ```
    ///
    /// At 60fps, `time_taken_since_last_frame_in_seconds` will be 1/60.
    ///
    /// Users of the Studio API should call
    /// [studio::System::set_listener_attributes] instead of this function.
    pub fn set_3d_listener_attributes(
        &self,
        listener: i32,
        attributes: ListenerAttributes3d,
    ) -> Result {
        ffi!(FMOD_System_Set3DListenerAttributes(
            self.as_raw(),
            listener,
            attributes.pos.as_raw(),
            attributes.vel.as_raw(),
            attributes.forward.as_raw(),
            attributes.up.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the position, velocity and orientation of the specified 3D sound listener.
    ///
    /// Users of the Studio API should call
    /// [studio::System::get_listener_attributes] instead of this function.
    pub fn get_3d_listener_attributes(&self, listener: i32) -> Result<ListenerAttributes3d> {
        let mut attributes = ListenerAttributes3d::default();
        ffi!(FMOD_System_Get3DListenerAttributes(
            self.as_raw(),
            listener,
            attributes.pos.as_raw_mut(),
            attributes.vel.as_raw_mut(),
            attributes.forward.as_raw_mut(),
            attributes.up.as_raw_mut(),
        ))?;
        Ok(attributes)
    }

    /// Sets parameters for the global reverb environment.
    ///
    /// To assist in defining reverb properties there are several presets
    /// available, see [ReverbProperties]' associated constants.
    ///
    /// When using each instance for the first time, FMOD will create a physical
    /// SFX reverb DSP unit that takes up several hundred kilobytes of memory
    /// and some CPU.
    pub fn set_reverb_properties(
        &self,
        instance: i32,
        properties: Option<&ReverbProperties>,
    ) -> Result {
        ffi!(FMOD_System_SetReverbProperties(
            self.as_raw(),
            instance,
            properties.map_or(ptr::null(), |x| x.as_raw()),
        ))?;
        Ok(())
    }

    /// Retrieves the current reverb environment for the specified reverb
    /// instance.
    pub fn get_reverb_properties(&self, instance: i32) -> Result<ReverbProperties> {
        let mut properties = ReverbProperties::default();
        ffi!(FMOD_System_GetReverbProperties(
            self.as_raw(),
            instance,
            properties.as_raw_mut(),
        ))?;
        Ok(properties)
    }

    /// Connect the output of the specified ChannelGroup to an audio port on the
    /// output driver.
    ///
    /// Ports are additional outputs supported by some [OutputType] plugins and
    /// can include things like controller headsets or dedicated background
    /// music streams. See the Port Support section (where applicable) of each
    /// platform's getting started guide found in the [platform details] chapter.
    ///
    /// [platform details]: https://fmod.com/resources/documentation-api?version=2.02&page=platforms.html
    pub fn attach_channel_group_to_port(
        &self,
        port_type: PortType,
        port_index: PortIndex,
        group: &ChannelGroup,
        pass_thru: bool,
    ) -> Result {
        ffi!(FMOD_System_AttachChannelGroupToPort(
            self.as_raw(),
            port_type.into_raw(),
            port_index.into_raw(),
            group.as_raw(),
            pass_thru as _,
        ))?;
        Ok(())
    }

    /// Disconnect the output of the specified ChannelGroup from an audio port
    /// on the output driver.
    ///
    /// Removing a [ChannelGroup] from a port will reroute the audio back to the
    /// main mix.
    pub fn detach_channel_group_from_port(&self, channel_group: &ChannelGroup) -> Result {
        ffi!(FMOD_System_DetachChannelGroupFromPort(
            self.as_raw(),
            channel_group.as_raw(),
        ))?;
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

fmod_struct! {
    /// Structure defining a reverb environment.
    ///
    /// The generic reverb properties are those used by [ReverbProperties::GENERIC].
    pub struct ReverbProperties = FMOD_REVERB_PROPERTIES {
        /// Reverberation decay time.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>1500</dd>
        /// <dt>Range</dt><dd>[0, 20000]</dd>
        /// </dl>
        #[default(1500.0)]
        pub decay_time: f32,
        /// Initial reflection delay time.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>7</dd>
        /// <dt>Range</dt><dd>[0, 300]</dd>
        /// </dl>
        #[default(7.0)]
        pub early_delay: f32,
        /// Late reverberation delay time relative to initial reflection.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>11</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(11.0)]
        pub late_delay: f32,
        /// Reference high frequency.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>5000</dd>
        /// <dt>Range</dt><dd>[20, 20000]</dd>
        /// </dl>
        #[default(5000.0)]
        pub hf_reference: f32,
        /// High-frequency to mid-frequency decay time ratio.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[10, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub hf_decay_ratio: f32,
        /// Value that controls the echo density in the late reverberation decay.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[10, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub diffusion: f32,
        /// Value that controls the modal density in the late reverberation decay.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>100</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(100.0)]
        pub density: f32,
        /// Reference low frequency
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>250</dd>
        /// <dt>Range</dt><dd>[20, 1000]</dd>
        /// </dl>
        #[default(250.0)]
        pub low_shelf_frequency: f32,
        /// Relative room effect level at low frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Decibels</dd>
        /// <dt>Default</dt><dd>0</dd>
        /// <dt>Range</dt><dd>[-36, 12]</dd>
        /// </dl>
        #[default(0.0)]
        pub low_shelf_gain: f32,
        /// Relative room effect level at high frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>200000</dd>
        /// <dt>Range</dt><dd>[0, 20000]</dd>
        /// </dl>
        #[default(200000.0)]
        pub high_cut: f32,
        /// Early reflections level relative to room effect.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub early_late_mix: f32,
        /// Room effect level at mid frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Decibels</dd>
        /// <dt>Default</dt><dd>-6</dd>
        /// <dt>Range</dt><dd>[-80, 20]</dd>
        /// </dl>
        #[default(-6.0)]
        pub wet_level: f32,
    }
}

// -------------------------------------------------------------------------------------------------

flags! {
    /// Output type specific index for when there are multiple instances of a port type.
    pub struct PortIndex: FMOD_PORT_INDEX {
        /// Use when a port index is not required
        None = FMOD_PORT_INDEX_NONE as _,
        /// Use as a flag to indicate the intended controller is associated with a VR headset
        VrController = FMOD_PORT_INDEX_FLAG_VR_CONTROLLER as _,
    }
}

enum_struct! {
    /// Port types available for routing audio.
    pub enum PortType: FMOD_PORT_TYPE {
        Music          = FMOD_PORT_TYPE_MUSIC,
        CopyrightMusic = FMOD_PORT_TYPE_COPYRIGHT_MUSIC,
        Voice          = FMOD_PORT_TYPE_VOICE,
        Controller     = FMOD_PORT_TYPE_CONTROLLER,
        Personal       = FMOD_PORT_TYPE_PERSONAL,
        Vibration      = FMOD_PORT_TYPE_VIBRATION,
        Aux            = FMOD_PORT_TYPE_AUX,
    }
}

/// The maximum number of global/physical reverb instances.
///
/// Each instance of a physical reverb is an instance of a [DspSfxReverb] dsp in
/// the mix graph. This is unrelated to the number of possible Reverb3D objects,
/// which is unlimited.
pub const REVERB_MAX_INSTANCES: usize = FMOD_REVERB_MAXINSTANCES as usize;

macro_rules! reverb {
    {
        $decay_time:expr,
        $early_delay:expr,
        $late_delay:expr,
        $hf_reference:expr,
        $hf_decay_ratio:expr,
        $diffusion:expr,
        $density:expr,
        $low_shelf_frequency:expr,
        $low_shelf_gain:expr,
        $high_cut:expr,
        $early_late_mix:expr,
        $wet_level:expr $(,)?
    } => {
        ReverbProperties {
            decay_time: $decay_time,
            early_delay: $early_delay,
            late_delay: $late_delay,
            hf_reference: $hf_reference,
            hf_decay_ratio: $hf_decay_ratio,
            diffusion: $diffusion,
            density: $density,
            low_shelf_frequency: $low_shelf_frequency,
            low_shelf_gain: $low_shelf_gain,
            high_cut: $high_cut,
            early_late_mix: $early_late_mix,
            wet_level: $wet_level,
        }
    };
}

#[rustfmt::skip]
impl ReverbProperties {
    pub const OFF: Self =               reverb! {  1000.0,    7.0,  11.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0,    20.0,  96.0, -80.0 };
    pub const GENERIC: Self =           reverb! {  1500.0,    7.0,  11.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0, 14500.0,  96.0,  -8.0 };
    pub const PADDEDCELL: Self =        reverb! {   170.0,    1.0,   2.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   160.0,  84.0,  -7.8 };
    pub const ROOM: Self =              reverb! {   400.0,    2.0,   3.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0,  6050.0,  88.0,  -9.4 };
    pub const BATHROOM: Self =          reverb! {  1500.0,    7.0,  11.0, 5000.0,  54.0, 100.0,  60.0, 250.0, 0.0,  2900.0,  83.0,   0.5 };
    pub const LIVINGROOM: Self =        reverb! {   500.0,    3.0,   4.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   160.0,  58.0, -19.0 };
    pub const STONEROOM: Self =         reverb! {  2300.0,   12.0,  17.0, 5000.0,  64.0, 100.0, 100.0, 250.0, 0.0,  7800.0,  71.0,  -8.5 };
    pub const AUDITORIUM: Self =        reverb! {  4300.0,   20.0,  30.0, 5000.0,  59.0, 100.0, 100.0, 250.0, 0.0,  5850.0,  64.0, -11.7 };
    pub const CONCERTHALL: Self =       reverb! {  3900.0,   20.0,  29.0, 5000.0,  70.0, 100.0, 100.0, 250.0, 0.0,  5650.0,  80.0,  -9.8 };
    pub const CAVE: Self =              reverb! {  2900.0,   15.0,  22.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0, 20000.0,  59.0, -11.3 };
    pub const ARENA: Self =             reverb! {  7200.0,   20.0,  30.0, 5000.0,  33.0, 100.0, 100.0, 250.0, 0.0,  4500.0,  80.0,  -9.6 };
    pub const HANGAR: Self =            reverb! { 10000.0,   20.0,  30.0, 5000.0,  23.0, 100.0, 100.0, 250.0, 0.0,  3400.0,  72.0,  -7.4 };
    pub const CARPETTEDHALLWAY: Self =  reverb! {   300.0,    2.0,  30.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   500.0,  56.0, -24.0 };
    pub const HALLWAY: Self =           reverb! {  1500.0,    7.0,  11.0, 5000.0,  59.0, 100.0, 100.0, 250.0, 0.0,  7800.0,  87.0,  -5.5 };
    pub const STONECORRIDOR: Self =     reverb! {   270.0,   13.0,  20.0, 5000.0,  79.0, 100.0, 100.0, 250.0, 0.0,  9000.0,  86.0,  -6.0 };
    pub const ALLEY: Self =             reverb! {  1500.0,    7.0,  11.0, 5000.0,  86.0, 100.0, 100.0, 250.0, 0.0,  8300.0,  80.0,  -9.8 };
    pub const FOREST: Self =            reverb! {  1500.0,  162.0,  88.0, 5000.0,  54.0,  79.0, 100.0, 250.0, 0.0,   760.0,  94.0, -12.3 };
    pub const CITY: Self =              reverb! {  1500.0,    7.0,  11.0, 5000.0,  67.0,  50.0, 100.0, 250.0, 0.0,  4050.0,  66.0, -26.0 };
    pub const MOUNTAINS: Self =         reverb! {  1500.0,  300.0, 100.0, 5000.0,  21.0,  27.0, 100.0, 250.0, 0.0,  1220.0,  82.0, -24.0 };
    pub const QUARRY: Self =            reverb! {  1500.0,   61.0,  25.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0,  3400.0, 100.0,  -5.0 };
    pub const PLAIN: Self =             reverb! {  1500.0,  179.0, 100.0, 5000.0,  50.0,  21.0, 100.0, 250.0, 0.0,  1670.0,  65.0, -28.0 };
    pub const PARKINGLOT: Self =        reverb! {  1700.0,    8.0,  12.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0, 20000.0,  56.0, -19.5 };
    pub const SEWERPIPE: Self =         reverb! {  2800.0,   14.0,  21.0, 5000.0,  14.0,  80.0,  60.0, 250.0, 0.0,  3400.0,  66.0,   1.2 };
    pub const UNDERWATER: Self =        reverb! {  1500.0,    7.0,  11.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   500.0,  92.0,   7.0 };
}

// -------------------------------------------------------------------------------------------------

/// Position, velocity, and orientation of a 3D sound listener.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ListenerAttributes3d {
    /// Position in 3D space used for panning and attenuation.
    pub pos: Vector,
    /// Velocity in 3D space used for doppler.
    pub vel: Vector,
    /// Forwards orientation.
    pub forward: Vector,
    /// Upwards orientation.
    pub up: Vector,
}

// -------------------------------------------------------------------------------------------------

/// # Recording.
impl System {
    /// Retrieves the number of recording devices available for this output
    /// mode. Use this to enumerate all recording devices possible so that the
    /// user can select one.
    pub fn get_record_num_drivers(&self) -> Result<NumDrivers> {
        let mut available = 0;
        let mut connected = 0;
        ffi!(FMOD_System_GetRecordNumDrivers(
            self.as_raw(),
            &mut available,
            &mut connected,
        ))?;
        Ok(NumDrivers {
            available,
            connected,
        })
    }

    /// Retrieves identification information about an audio device specified by
    /// its index, and specific to the output mode.
    pub fn get_record_driver_info(&self, id: i32) -> Result<DriverInfo> {
        let mut guid = Guid::default();
        let mut system_rate = 0;
        let mut speaker_mode = SpeakerMode::default();
        let mut speaker_mode_channels = 0;
        let mut state = DriverState::default();

        ffi!(FMOD_System_GetRecordDriverInfo(
            self.as_raw(),
            id,
            ptr::null_mut(),
            0,
            guid.as_raw_mut(),
            &mut system_rate,
            speaker_mode.as_raw_mut(),
            &mut speaker_mode_channels,
            state.as_raw_mut(),
        ))?;

        Ok(DriverInfo {
            guid,
            system_rate,
            speaker_mode,
            speaker_mode_channels,
            state,
        })
    }

    /// Retrieves the name of an audio device specified by its index, and
    /// specific to the output mode.
    pub fn get_record_driver_name(&self, id: i32, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_System_GetDriverInfo(
                    self.as_raw(),
                    id,
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ))
            })
        }
    }

    /// Retrieves the current recording position of the record buffer in PCM
    /// samples.
    ///
    /// Will return [`Error::RecordDisconnected`] if the driver is unplugged.
    ///
    /// The position will return to 0 when [`System::record_stop`] is called or
    /// when a non-looping recording reaches the end.
    pub fn get_record_position(&self, id: i32) -> Result<Time> {
        let mut position = 0;
        ffi!(FMOD_System_GetRecordPosition(
            self.as_raw(),
            id,
            &mut position,
        ))?;
        Ok(Time::pcm(position))
    }

    /// Starts the recording engine recording to a pre-created Sound object.
    ///
    /// Will return [`Error::RecordDisconnected`] if the driver is unplugged.
    ///
    /// Sound must be created as [`Mode::CreateSample`]. Raw PCM data can be
    /// accessed with [`Sound::lock`], [`Sound::unlock`] and
    /// [`System::get_record_position`].
    ///
    /// Recording from the same driver a second time will stop the first
    /// recording.
    ///
    /// For lowest latency set the Sound sample rate to the rate returned by
    /// [`System::get_record_driver_info`], otherwise a resampler will be
    /// allocated to handle the difference in frequencies, which adds latency.
    pub fn record_start(&self, id: i32, sound: &Sound) -> Result {
        ffi!(FMOD_System_RecordStart(
            self.as_raw(),
            id,
            sound.as_raw(),
            false as _, // loop
        ))?;
        Ok(())
    }

    /// Starts the recording engine recording to a pre-created Sound object.
    ///
    /// Like [`System::record_start`], but the recording engine will continue
    /// recording to the provided sound from the start again, after it has
    /// reached the end. The data will be continually be overwritten once every
    /// loop.
    pub fn record_start_loop(&self, id: i32, sound: &Sound) -> Result {
        ffi!(FMOD_System_RecordStart(
            self.as_raw(),
            id,
            sound.as_raw(),
            true as _, // loop
        ))?;
        Ok(())
    }

    /// Stops the recording engine from recording to a pre-created Sound object.
    ///
    /// Returns no error if unplugged or already stopped.
    pub fn record_stop(&self, id: i32) -> Result {
        ffi!(FMOD_System_RecordStop(self.as_raw(), id))?;
        Ok(())
    }

    /// Retrieves the state of the FMOD recording API, i.e. if it is currently
    /// recording or not.
    ///
    /// Recording can be started with [`System::record_start`] and stopped with
    /// [`System::record_stop`].
    ///
    /// Will return [`Error::RecordDisconnected`] if the driver is unplugged.
    pub fn is_recording(&self, id: i32) -> Result<bool> {
        let mut recording = 0;
        ffi!(FMOD_System_IsRecording(self.as_raw(), id, &mut recording))?;
        Ok(recording != 0)
    }
}

// -------------------------------------------------------------------------------------------------

enum_struct! {
    /// Flags that provide additional information about a particular driver.
    pub enum DriverState: FMOD_DRIVER_STATE {
        /// Device is currently plugged in.
        Connected = FMOD_DRIVER_STATE_CONNECTED,
        #[default]
        /// Device is the users preferred choice.
        Default   = FMOD_DRIVER_STATE_DEFAULT,
    }
}

// -------------------------------------------------------------------------------------------------

/// Number of recording devices available.
#[derive(Debug)]
pub struct NumDrivers {
    /// Number of recording drivers available for this output mode.
    pub available: i32,
    /// Number of recording driver currently plugged in.
    pub connected: i32,
}

// -------------------------------------------------------------------------------------------------

/// # Geometry management.
impl System {
    /// Geometry creation function. This function will create a base geometry
    /// object which can then have polygons added to it.
    ///
    /// Polygons can be added to a geometry object using
    /// [`Geometry::add_polygon`]. For best efficiency, avoid overlapping of
    /// polygons and long thin polygons.
    ///
    /// A geometry object stores its polygons in a group to allow optimization
    /// for line testing, insertion and updating of geometry in real-time.
    /// Geometry objects also allow for efficient rotation, scaling and
    /// translation of groups of polygons.
    ///
    /// It is important to set the value of max_world_size to an appropriate
    /// value using [`System::set_geometry_settings`].
    pub fn create_geometry(
        &self,
        max_polygons: i32,
        max_vertices: i32,
    ) -> Result<Handle<'_, Geometry>> {
        let mut geometry = ptr::null_mut();
        ffi!(FMOD_System_CreateGeometry(
            self.as_raw(),
            max_polygons,
            max_vertices,
            &mut geometry,
        ))?;
        Ok(unsafe { Handle::new(geometry) })
    }

    /// Sets the maximum world size for the geometry engine for performance /
    /// precision reasons.
    ///
    /// FMOD uses an efficient spatial partitioning system to store polygons for
    /// ray casting purposes. The maximum size of the world should be set to
    /// allow processing within a known range. Outside of this range, objects
    /// and polygons will not be processed as efficiently. Excessive world size
    /// settings can also cause loss of precision and efficiency.
    ///
    /// Setting `max_world_size` should be done first before creating any
    /// geometry. It can be done any time afterwards but may be slow in this
    /// case.
    pub fn set_geometry_settings(&self, max_world_size: f32) -> Result {
        ffi!(FMOD_System_SetGeometrySettings(
            self.as_raw(),
            max_world_size,
        ))?;
        Ok(())
    }

    /// Retrieves the maximum world size for the geometry engine.
    ///
    /// FMOD uses an efficient spatial partitioning system to store polygons for
    /// ray casting purposes. The maximum size of the world should be set to
    /// allow processing within a known range. Outside of this range, objects
    /// and polygons will not be processed as efficiently. Excessive world size
    /// settings can also cause loss of precision and efficiency.
    pub fn get_geometry_settings(&self) -> Result<f32> {
        let mut max_world_size = 0.0;
        ffi!(FMOD_System_GetGeometrySettings(
            self.as_raw(),
            &mut max_world_size,
        ))?;
        Ok(max_world_size)
    }

    /// Creates a geometry object from a block of memory which contains
    /// pre-saved geometry data from [`Geometry::save`].
    ///
    /// This function avoids the need to manually create and add geometry for
    /// faster start time.
    pub fn load_geometry(&self, data: &[u8]) -> Result<Handle<'_, Geometry>> {
        let mut geometry = ptr::null_mut();
        ffi!(FMOD_System_LoadGeometry(
            self.as_raw(),
            data.as_ptr().cast(),
            data.len() as _,
            &mut geometry,
        ))?;
        Ok(unsafe { Handle::new(geometry) })
    }

    /// Calculates geometry occlusion between a listener and a sound source.
    ///
    /// If single sided polygons have been created, it is important to get the
    /// source and listener positions around the right way, as the occlusion
    /// from point A to point B may not be the same as the occlusion from point
    /// B to point A.
    pub fn get_geometry_occlusion(&self, listener: &Vector, source: &Vector) -> Result<Occlusion> {
        let mut direct = 0.0;
        let mut reverb = 0.0;
        ffi!(FMOD_System_GetGeometryOcclusion(
            self.as_raw(),
            listener.as_raw(),
            source.as_raw(),
            &mut direct,
            &mut reverb,
        ))?;
        Ok(Occlusion { direct, reverb })
    }
}

// -------------------------------------------------------------------------------------------------

/// # General.
impl System {
    /// Mutual exclusion function to lock the FMOD DSP engine (which runs
    /// asynchronously in another thread), so that it will not execute.
    ///
    /// If the FMOD DSP engine is already executing, this function will block
    /// until it has completed.
    ///
    /// The function may be used to synchronize DSP network operations carried
    /// out by the user.
    ///
    /// An example of using this function may be for when the user wants to
    /// construct a DSP sub-network, without the DSP engine executing in the
    /// background while the sub-network is still under construction.
    ///
    /// Once the user no longer needs the DSP engine locked, it must be unlocked
    /// by dropping the returned [`DspLock`] or calling [`System::unlock_dsp`].
    ///
    /// Note that the DSP engine should not be locked for a significant amount
    /// of time, otherwise inconsistency in the audio output may result.
    /// (audio skipping / stuttering).
    ///
    /// # Safety
    ///
    /// The DSP engine must not already be locked when this function is called.
    pub unsafe fn lock_dsp(&self) -> Result<DspLock<'_>> {
        ffi!(FMOD_System_LockDSP(self.as_raw()))?;
        Ok(DspLock { system: self })
    }

    /// Mutual exclusion function to unlock the FMOD DSP engine (which runs
    /// asynchronously in another thread) and let it continue executing.
    ///
    /// # Safety
    ///
    /// The DSP engine must be locked with [`System::lock_dsp`] before this
    /// function is called and the [`DspLock`] guard forgotten.
    pub unsafe fn unlock_dsp(&self) -> Result {
        ffi!(FMOD_System_UnlockDSP(self.as_raw()))?;
        Ok(())
    }

    /// Sets the callback for System level notifications.
    ///
    /// Using [`SystemCallbackType::ALL`] or
    /// [`SystemCallbackType:DeviceListChanged`] will disable any automated
    /// device ejection/insertion handling. Use this callback to control the
    /// behavior yourself.
    ///
    /// Using [`SystemCallbackType:DeviceListChanged`] (Mac only) requires the
    /// application to be running an event loop which will allow external
    /// changes to device list to be detected.
    pub fn set_callback<C: SystemCallback>(&self, mask: SystemCallbackType) -> Result {
        ffi!(FMOD_System_SetCallback(
            self.as_raw(),
            Some(system_callback::<C>),
            mask.into_raw(),
        ))?;
        Ok(())
    }

    // set_user_data, get_user_data
}

// -------------------------------------------------------------------------------------------------

fmod_struct! {
    #![fmod_no_default]
    /// Information describing an error that has occurred.
    pub struct ErrorInfo<'a> = FMOD_ERRORCALLBACK_INFO {
        result: Result,
        instance_type: InstanceType,
        instance: *mut c_void,
        function_name: *const c_char,
        function_params: *const c_char,
        marker: PhantomData<&'a ()>,
    }
}

impl ErrorInfo<'_> {
    pub fn error(&self) -> Error {
        self.result.expect_err("error should have errored")
    }

    /// The fmod object instance that the error occurred on.
    pub fn instance(&self) -> Instance<'_> {
        macro_rules! map {
            (studio::$ty:ident) => {
                paste::paste! {
                    if let InstanceType::[<Studio $ty>] = self.instance_type {
                        return Instance::[<Studio $ty>](unsafe { studio::$ty::from_raw(self.instance.cast()) });
                    }
                }
            };
            ($ty:ident) => {
                if let InstanceType::$ty = self.instance_type {
                    return Instance::$ty(unsafe { $ty::from_raw(self.instance.cast()) });
                }
            };
        }

        map!(System);
        map!(Channel);
        map!(ChannelGroup);
        map!(ChannelControl);
        map!(Sound);
        map!(SoundGroup);
        map!(Dsp);
        map!(DspConnection);
        map!(Geometry);
        map!(Reverb3d);
        // #[cfg(feature = "studio")]
        // {
        //     map!(studio::System);
        //     map!(studio::EventDescription);
        //     map!(studio::EventInstance);
        //     map!(studio::Bus);
        //     map!(studio::Vca);
        //     map!(studio::Bank);
        //     map!(studio::CommandReplay);
        // }

        whoops!("unknown/unmapped instance type: {:?}", self.instance_type);
        Instance::Unknown
    }

    /// Function that the error occurred on.
    pub fn function_name(&self) -> Cow<'_, str> {
        debug_assert!(!self.function_name.is_null());
        unsafe { CStr::from_ptr(self.function_name) }.to_string_lossy()
    }

    /// Function parameters that the error ocurred on.
    pub fn function_params(&self) -> Cow<'_, str> {
        debug_assert!(!self.function_params.is_null());
        unsafe { CStr::from_ptr(self.function_params) }.to_string_lossy()
    }
}

#[cfg(windows)]
pub type SysThreadHandle = std::os::windows::io::RawHandle;
#[cfg(unix)]
pub type SysThreadHandle = std::os::unix::thread::RawPthread;

pub trait SystemCallback {
    /// Called from [`System::update`] when the enumerated list of devices has
    /// changed. Called from the main (calling) thread when set from the Core
    /// API or Studio API in synchronous mode, and from the Studio Update Thread
    /// when in default / async mode.
    fn device_list_changed(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called directly when a memory allocation fails.
    fn memory_allocation_failed(system: &System, location: &str, size: i32) -> Result {
        let _ = (system, location, size);
        Ok(())
    }

    /// Called from the game thread when a thread is created.
    fn thread_created(system: &System, thread: SysThreadHandle, name: &str) -> Result {
        let _ = (system, thread, name);
        Ok(())
    }

    /// Called from the mixer thread before it starts the next block.
    fn pre_mix(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called from the mixer thread after it finishes a block.
    fn post_mix(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called directly when an API function returns an error,
    /// including delayed async functions.
    fn error(system: &System, info: &ErrorInfo<'_>) -> Result {
        let _ = (system, info);
        Ok(())
    }

    /// Called from the mixer thread after clocks have been updated before the main mix occurs.
    fn mid_mix(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called from the game thread when a thread is destroyed.
    fn thread_destroyed(system: &System, thread: SysThreadHandle, name: &str) -> Result {
        let _ = (system, thread, name);
        Ok(())
    }

    /// Called at start of [`System::update`] from the main (calling) thread
    /// when set from the Core API or Studio API in synchronous mode, and from
    /// the Studio Update Thread when in default / async mode.
    fn pre_update(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called at end of [`System::update`] from the main (calling) thread when
    /// set from the Core API or Studio API in synchronous mode, and from the
    /// Studio Update Thread when in default / async mode.
    fn post_update(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called from [`System::update`] when the enumerated list of recording
    /// devices has changed. Called from the main (calling) thread when set
    /// from the Core API or Studio API in synchronous mode, and from the
    /// Studio Update Thread when in default / async mode.
    fn record_list_changed(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called from the feeder thread after audio was consumed from the ring
    /// buffer, but not enough to allow another mix to run.
    fn buffered_no_mix(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called from System::update when an output device is re-initialized.
    /// Called from the main (calling) thread when set from the Core API or
    /// Studio API in synchronous mode, and from the Studio Update Thread when
    /// in default / async mode.
    fn device_reinitialize(system: &System, kind: OutputType, id: i32) -> Result {
        let _ = (system, kind, id);
        Ok(())
    }

    /// Called from the mixer thread when the device output attempts to read
    /// more samples than are available in the output buffer.
    fn output_underrun(system: &System) -> Result {
        let _ = system;
        Ok(())
    }

    /// Called from the mixer thread when the System record position changed.
    fn record_position_changed(system: &System, sound: &Sound, position: Time) -> Result {
        let _ = (system, sound, position);
        Ok(())
    }
}

pub(crate) unsafe extern "system" fn system_callback<C: SystemCallback>(
    system: *mut FMOD_SYSTEM,
    kind: FMOD_SYSTEM_CALLBACK_TYPE,
    commanddata1: *mut c_void,
    commanddata2: *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let kind = SystemCallbackType::from_raw(kind);
    let system = System::from_raw(system);
    catch_user_unwind(|| match kind {
        SystemCallbackType::DeviceListChanged => C::device_list_changed(system),
        SystemCallbackType::MemoryAllocationFailed => {
            let location = CStr::from_ptr(commanddata1.cast()).to_string_lossy();
            let size = commanddata2.cast::<i32>().read();
            C::memory_allocation_failed(system, &location, size)
        },
        SystemCallbackType::ThreadCreated => {
            let thread = commanddata1 as SysThreadHandle;
            let name = CStr::from_ptr(commanddata2.cast()).to_string_lossy();
            C::thread_created(system, thread, &name)
        },
        SystemCallbackType::PreMix => C::pre_mix(system),
        SystemCallbackType::PostMix => C::post_mix(system),
        SystemCallbackType::Error => {
            C::error(system, ErrorInfo::from_raw_ref(&*(commanddata1.cast())))
        },
        SystemCallbackType::MidMix => C::mid_mix(system),
        SystemCallbackType::ThreadDestroyed => {
            let thread = commanddata1 as SysThreadHandle;
            let name = CStr::from_ptr(commanddata2.cast()).to_string_lossy();
            C::thread_destroyed(system, thread, &name)
        },
        SystemCallbackType::PreUpdate => C::pre_update(system),
        SystemCallbackType::PostUpdate => C::post_update(system),
        SystemCallbackType::RecordListChanged => C::record_list_changed(system),
        SystemCallbackType::BufferedNoMix => C::buffered_no_mix(system),
        SystemCallbackType::DeviceReinitialize => {
            let kind = OutputType::from_raw(commanddata1 as _);
            let id = commanddata2.cast::<i32>().read();
            C::device_reinitialize(system, kind, id)
        },
        SystemCallbackType::OutputUnderrun => C::output_underrun(system),
        SystemCallbackType::RecordPositionChanged => {
            let sound = Sound::from_raw(commanddata1.cast());
            let position = Time::pcm(commanddata2 as _);
            C::record_position_changed(system, sound, position)
        },
        _ => {
            whoops!(no_panic: "unknown system callback type: {kind:?}");
            yeet!(Error::InvalidParam);
        },
    })
    .into_raw()
}

// -------------------------------------------------------------------------------------------------

raw! {
    enum_struct! {
        /// Identifier used to represent the different types of instance in the error callback.
        pub enum InstanceType: FMOD_ERRORCALLBACK_INSTANCETYPE {
            /// Type representing no known instance type.
            None                    = FMOD_ERRORCALLBACK_INSTANCETYPE_NONE,
            /// Type representing [System].
            System                  = FMOD_ERRORCALLBACK_INSTANCETYPE_SYSTEM,
            /// Type representing [Channel].
            Channel                 = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNEL,
            /// Type representing [ChannelGroup].
            ChannelGroup            = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNELGROUP,
            /// Type representing [ChannelControl].
            ChannelControl          = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNELCONTROL,
            /// Type representing [Sound].
            Sound                   = FMOD_ERRORCALLBACK_INSTANCETYPE_SOUND,
            /// Type representing [SoundGroup].
            SoundGroup              = FMOD_ERRORCALLBACK_INSTANCETYPE_SOUNDGROUP,
            /// Type representing [Dsp].
            Dsp                     = FMOD_ERRORCALLBACK_INSTANCETYPE_DSP,
            /// Type representing [DspConnection].
            DspConnection           = FMOD_ERRORCALLBACK_INSTANCETYPE_DSPCONNECTION,
            /// Type representing [Geometry].
            Geometry                = FMOD_ERRORCALLBACK_INSTANCETYPE_GEOMETRY,
            /// Type representing [Reverb3d].
            Reverb3d                = FMOD_ERRORCALLBACK_INSTANCETYPE_REVERB3D,
            /// Type representing [studio::System].
            StudioSystem            = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_SYSTEM,
            /// Type representing [studio::EventDescription].
            StudioEventDescription  = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_EVENTDESCRIPTION,
            /// Type representing [studio::EventInstance].
            StudioEventInstance     = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_EVENTINSTANCE,
            /// Deprecated.
            #[deprecated]
            StudioParameterInstance = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_PARAMETERINSTANCE,
            /// Type representing [studio::Bus].
            StudioBus               = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_BUS,
            /// Type representing [studio::Vca].
            StudioVca               = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_VCA,
            /// Type representing [studio::Bank].
            StudioBank              = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_BANK,
            /// Type representing [studio::CommandReplay].
            StudioCommandReplay     = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_COMMANDREPLAY,
        }
    }
}

#[non_exhaustive]
pub enum Instance<'a> {
    #[doc(hidden)]
    Unknown,
    System(&'a System),
    Channel(&'a Channel),
    ChannelGroup(&'a ChannelGroup),
    ChannelControl(&'a ChannelControl),
    Sound(&'a Sound),
    SoundGroup(&'a SoundGroup),
    Dsp(&'a Dsp),
    DspConnection(&'a DspConnection),
    Geometry(&'a Geometry),
    Reverb3d(&'a Reverb3d),
    // #[cfg(feature = "studio")]
    // StudioSystem(&'a studio::System),
    // #[cfg(feature = "studio")]
    // StudioEventDescription(&'a studio::EventDescription),
    // #[cfg(feature = "studio")]
    // StudioEventInstance(&'a studio::EventInstance),
    // #[cfg(feature = "studio")]
    // StudioBus(&'a studio::Bus),
    // #[cfg(feature = "studio")]
    // StudioVca(&'a studio::Vca),
    // #[cfg(feature = "studio")]
    // StudioBank(&'a studio::Bank),
    // #[cfg(feature = "studio")]
    // StudioCommandReplay(&'a studio::CommandReplay),
}

flags! {
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
        RecordPositionChanged  = FMOD_SYSTEM_CALLBACK_RECORDPOSITIONCHANGED,
        /// Mask representing all callback types.
        All                    = FMOD_SYSTEM_CALLBACK_ALL,
    }
}

// -------------------------------------------------------------------------------------------------

/// Mutual exclusion lock for the FMOD DSP engine.
pub struct DspLock<'a> {
    system: &'a System,
}

impl DspLock<'_> {
    pub fn unlock(self) -> Result {
        let this = ManuallyDrop::new(self);
        unsafe { this.system.unlock_dsp() }
    }
}

impl Drop for DspLock<'_> {
    fn drop(&mut self) {
        match unsafe { self.system.unlock_dsp() } {
            Ok(()) => (),
            Err(e) => {
                whoops!("Error unlocking DSP engine: {e}");
            },
        }
    }
}
