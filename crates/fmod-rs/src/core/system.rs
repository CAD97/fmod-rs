use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    parking_lot::RwLockUpgradableReadGuard,
    smart_default::SmartDefault,
    std::{ffi::CStr, mem, ptr, time::Duration},
};

opaque! {
    /// Management object from which all resources are created and played.
    class System = FMOD_SYSTEM, FMOD_System_* (System::raw_release);
}

/// Lifetime management.
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
            return Err(Error::Initialized);
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
        fmod_try!(FMOD_System_Create(&mut raw, FMOD_VERSION));
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
        fmod_try!(FMOD_System_Init(
            self.as_raw(),
            max_channels,
            flags,
            extra_driver_data as *mut _,
        ));
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
        fmod_try!(FMOD_System_Close(self.as_raw()));
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
        fmod_try!(FMOD_System_Update(self.as_raw()));
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
        fmod_try!(FMOD_System_MixerSuspend(self.as_raw()));
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
        fmod_try!(FMOD_System_MixerResume(self.as_raw()));
        Ok(())
    }
}

/// Identification information about a sound device specified by its index,
/// specific to the selected output mode.
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
}

/// Device selection.
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
        fmod_try!(FMOD_System_SetOutput(self.as_raw(), output));
        Ok(())
    }

    /// Retrieves the type of output interface used to run the mixer.
    pub fn get_output(&self) -> Result<fmod::OutputType> {
        let mut output = OutputType::zeroed();
        fmod_try!(FMOD_System_GetOutput(self.as_raw(), output.as_raw_mut()));
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
        fmod_try!(FMOD_System_GetNumDrivers(self.as_raw(), &mut numdrivers));
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

        fmod_try!(FMOD_System_GetDriverInfo(
            self.as_raw(),
            id,
            ptr::null_mut(),
            0,
            guid.as_raw_mut(),
            &mut system_rate,
            speaker_mode.as_raw_mut(),
            &mut speaker_mode_channels,
        ));

        Ok(DriverInfo {
            guid,
            system_rate,
            speaker_mode,
            speaker_mode_channels,
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
                Error::from_raw(FMOD_System_GetDriverInfo(
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
        fmod_try!(FMOD_System_SetDriver(self.as_raw(), id));
        Ok(())
    }

    /// Retrieves the output driver index for the selected output type.
    ///
    /// 0 represents the default for the output type.
    pub fn get_driver(&self) -> Result<i32> {
        let mut driver = 0;
        fmod_try!(FMOD_System_GetDriver(self.as_raw(), &mut driver));
        Ok(driver)
    }
}

/// Output format for the software mixer.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SoftwareFormat {
    /// Sample rate of the mixer.
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[8000, 192000]</dd>
    /// <dt>Units</dt><dd>Hertz</dd>
    /// <dt>Default</dt><dd>48000</dd>
    pub sample_rate: i32,
    /// Speaker setup of the mixer.
    pub speaker_mode: SpeakerMode,
    /// Number of speakers for [SpeakerMode::Raw].
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[0, MAX_CHANNEL_WIDTH]</dd>
    /// </d>
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

/// Callback to allow custom calculation of distance attenuation.
pub type Rolloff3dCallback = extern "C" fn(channel: &Channel, distance: f32) -> f32;

/// Setup.
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
        fmod_try!(FMOD_System_SetSoftwareChannels(
            self.as_raw(),
            num_software_channels,
        ));
        Ok(())
    }

    /// Retrieves the maximum number of software mixed channels possible.
    pub fn get_software_channels(&self) -> Result<i32> {
        let mut num_software_channels = 0;
        fmod_try!(FMOD_System_GetSoftwareChannels(
            self.as_raw(),
            &mut num_software_channels,
        ));
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
        fmod_try!(FMOD_System_SetSoftwareFormat(
            self.as_raw(),
            sample_rate,
            speaker_mode.into_raw(),
            num_raw_speakers,
        ));
        Ok(())
    }

    /// Retrieves the output format for the software mixer.
    pub fn get_software_format(&self) -> Result<SoftwareFormat> {
        let mut sample_rate = 0;
        let mut speaker_mode = SpeakerMode::default();
        let mut num_raw_speakers = 0;
        fmod_try!(FMOD_System_GetSoftwareFormat(
            self.as_raw(),
            &mut sample_rate,
            speaker_mode.as_raw_mut(),
            &mut num_raw_speakers
        ));
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
        fmod_try!(FMOD_System_SetDSPBufferSize(
            self.as_raw(),
            buffer_length,
            num_buffers,
        ));
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
        fmod_try!(FMOD_System_GetDSPBufferSize(
            self.as_raw(),
            &mut bufferlength,
            &mut numbuffers,
        ));
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
    pub fn set_stream_buffer_size(
        &self,
        file_buffer_size: u32,
        file_buffer_size_type: TimeUnit,
    ) -> Result {
        fmod_try!(FMOD_System_SetStreamBufferSize(
            self.as_raw(),
            file_buffer_size,
            file_buffer_size_type.into_raw(),
        ));
        Ok(())
    }

    /// Retrieves the default file buffer size for newly opened streams.
    ///
    /// Valid units are [TimeUnit::Ms], [Pcm](TimeUnit::Pcm),
    /// [PcmBytes](TimeUnit::PcmBytes), and [RawBytes](TimeUnit::RawBytes).
    pub fn get_stream_buffer_size(&self) -> Result<(u32, TimeUnit)> {
        let mut file_buffer_size = 0;
        let mut file_buffer_size_type = TimeUnit::zeroed();
        fmod_try!(FMOD_System_GetStreamBufferSize(
            self.as_raw(),
            &mut file_buffer_size,
            file_buffer_size_type.as_raw_mut(),
        ));
        Ok((file_buffer_size, file_buffer_size_type))
    }

    /// Sets advanced settings for the system object, typically to allow
    /// adjusting of settings related to resource usage or audio quality.
    pub fn set_advanced_settings(&self, mut advanced_settings: AdvancedSettings) -> Result {
        fmod_try!(FMOD_System_SetAdvancedSettings(
            self.as_raw(),
            advanced_settings.as_raw_mut(),
        ));
        Ok(())
    }

    /// Retrieves the advanced settings for the system object.
    pub fn get_advanced_settings(&self) -> Result<AdvancedSettings> {
        let mut advanced_settings = AdvancedSettings::default();
        fmod_try!(FMOD_System_GetAdvancedSettings(
            self.as_raw(),
            advanced_settings.as_raw_mut(),
        ));
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
        fmod_try!(FMOD_System_SetSpeakerPosition(
            self.as_raw(),
            speaker.into_raw(),
            x,
            y,
            if active { 1 } else { 0 },
        ));
        Ok(())
    }

    /// Retrieves the position of the specified speaker for the current speaker
    /// mode.
    pub fn get_speaker_position(&self, speaker: Speaker) -> Result<SpeakerPosition> {
        let mut speaker_position = SpeakerPosition::default();
        let mut active = 0;
        fmod_try!(FMOD_System_GetSpeakerPosition(
            self.as_raw(),
            speaker.into_raw(),
            &mut speaker_position.x,
            &mut speaker_position.y,
            &mut active,
        ));
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
        fmod_try!(FMOD_System_Set3DSettings(
            self.as_raw(),
            doppler_scale,
            distance_factor,
            rolloff_scale,
        ));
        Ok(())
    }

    /// Retrieves the global doppler scale, distance factor and rolloff scale for all 3D sounds.
    pub fn get_3d_settings(&self) -> Result<Settings3d> {
        let mut settings = Settings3d::default();
        fmod_try!(FMOD_System_Get3DSettings(
            self.as_raw(),
            &mut settings.doppler_scale,
            &mut settings.distance_factor,
            &mut settings.rolloff_scale,
        ));
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
        fmod_try!(FMOD_System_Set3DNumListeners(self.as_raw(), num_listeners));
        Ok(())
    }

    /// Retrieves the number of 3D listeners.
    ///
    /// Users of the Studio API should call [studio::System::get_num_listeners]
    /// instead of this function.
    pub fn get_3d_num_listeners(&self) -> Result<i32> {
        let mut num_listeners = 0;
        fmod_try!(FMOD_System_Get3DNumListeners(
            self.as_raw(),
            &mut num_listeners,
        ));
        Ok(num_listeners)
    }

    /// Sets a callback to allow custom calculation of distance attenuation.
    ///
    /// This function overrides [Mode::InverseRolloff3d],
    /// [Mode::LinearRolloff3d], [Mode::LinearSquareRolloff3d],
    /// [Mode::InverseTaperedRolloff3d], and [Mode::CustomRolloff3d].
    pub fn set_3d_rolloff_callback(&self, callback: Option<Rolloff3dCallback>) -> Result {
        fmod_try!(FMOD_System_Set3DRolloffCallback(
            self.as_raw(),
            mem::transmute(callback),
        ));
        Ok(())
    }
}

/// File system setup.
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
        fmod_try!(FMOD_System_SetFileSystem(
            self.as_raw(),
            None,
            None,
            None,
            None,
            None,
            None,
            block_align,
        ));
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
        fmod_try!(FMOD_System_SetFileSystem(
            self.as_raw(),
            Some(file::useropen::<FS>),
            Some(file::userclose::<FS>),
            Some(file::userread::<FS>),
            Some(file::userseek::<FS>),
            None,
            None,
            block_align,
        ));
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
        fmod_try!(FMOD_System_SetFileSystem(
            self.as_raw(),
            Some(file::useropen::<FS>),
            Some(file::userclose::<FS>),
            None,
            None,
            Some(file::userasyncread::<FS>),
            Some(file::userasynccancel::<FS>),
            block_align,
        ));
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
        fmod_try!(FMOD_System_AttachFileSystem(
            self.as_raw(),
            Some(file::useropen_listen::<FS>),
            Some(file::userclose_listen::<FS>),
            Some(file::userread_listen::<FS>),
            Some(file::userseek_listen::<FS>),
        ));
        Ok(())
    }

    /// Detach a previously [attached](Self::attach_file_system) file system
    /// listener.
    pub fn detach_file_system(&self) -> Result {
        fmod_try!(FMOD_System_AttachFileSystem(
            self.as_raw(),
            None,
            None,
            None,
            None,
        ));
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PluginHandle {
    raw: u32,
}

impl PluginHandle {
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub const unsafe fn from_raw(raw: u32) -> Self {
            Self { raw }
        }
    }
    raw! {
        pub const fn from_raw_ref(raw: &u32) -> &Self {
            unsafe { &*(raw as *const u32 as *const Self ) }
        }
    }
    raw! {
        pub fn from_raw_mut(raw: &mut u32) -> &mut Self {
            unsafe { &mut *(raw as *mut u32 as *mut Self ) }
        }
    }
    raw! {
        pub const fn into_raw(self) -> u32 {
            self.raw
        }
    }
    raw! {
        pub const fn as_raw(&self) -> &u32 {
            unsafe { &*(self as *const Self as *const u32 ) }
        }
    }
    raw! {
        pub fn as_raw_mut(&mut self) -> &mut u32 {
            unsafe { &mut *(self as *mut Self as *mut u32 ) }
        }
    }
}

/// Information about a selected plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// Plugin type.
    pub kind: PluginType,
    /// Version number of the plugin.
    pub version: u32,
}

/// Plugin support.
impl System {
    /// Specify a base search path for plugins so they can be placed somewhere
    /// else than the directory of the main executable.
    pub fn set_plugin_path(&self, path: &CStr) -> Result {
        fmod_try!(FMOD_System_SetPluginPath(self.as_raw(), path.as_ptr()));
        Ok(())
    }

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
    pub fn load_plugin(&self, filename: &CStr, priority: u32) -> Result<PluginHandle> {
        let mut handle = PluginHandle::default();
        fmod_try!(FMOD_System_LoadPlugin(
            self.as_raw(),
            filename.as_ptr(),
            handle.as_raw_mut(),
            priority,
        ));
        Ok(handle)
    }

    /// Unloads an FMOD (DSP, Output or Codec) plugin.
    pub fn unload_plugin(&self, handle: PluginHandle) -> Result {
        fmod_try!(FMOD_System_UnloadPlugin(self.as_raw(), handle.into_raw()));
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
    pub fn get_num_tested_plugins(&self, handle: PluginHandle) -> Result<i32> {
        let mut count = 0;
        fmod_try!(FMOD_System_GetNumNestedPlugins(
            self.as_raw(),
            handle.into_raw(),
            &mut count,
        ));
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
        fmod_try!(FMOD_System_GetNestedPlugin(
            self.as_raw(),
            handle.into_raw(),
            index,
            nested_handle.as_raw_mut(),
        ));
        Ok(nested_handle)
    }

    /// Retrieves the number of loaded plugins.
    pub fn get_num_plugins(&self, plugin_type: PluginType) -> Result<i32> {
        let mut num_plugins = 0;
        fmod_try!(FMOD_System_GetNumPlugins(
            self.as_raw(),
            plugin_type.into_raw(),
            &mut num_plugins,
        ));
        Ok(num_plugins)
    }

    /// Retrieves the handle of a plugin based on its type and relative index.
    ///
    /// All plugins whether built in or loaded can be enumerated using this and
    /// [System::get_num_plugins].
    pub fn get_plugin_handle(&self, plugin_type: PluginType, index: i32) -> Result<PluginHandle> {
        let mut handle = PluginHandle::default();
        fmod_try!(FMOD_System_GetPluginHandle(
            self.as_raw(),
            plugin_type.into_raw(),
            index,
            handle.as_raw_mut(),
        ));
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
        fmod_try!(FMOD_System_GetPluginInfo(
            self.as_raw(),
            handle.raw,
            kind.as_raw_mut(),
            ptr::null_mut(),
            0,
            &mut version,
        ));
        Ok(PluginInfo { kind, version })
    }

    /// Retrieves name for the selected plugin.
    pub fn get_plugin_name(&self, handle: PluginHandle, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                Error::from_raw(FMOD_System_GetPluginInfo(
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
        fmod_try!(FMOD_System_SetOutputByPlugin(
            self.as_raw(),
            handle.into_raw(),
        ));
        Ok(())
    }

    /// Retrieves the plugin handle for the currently selected output type.
    pub fn get_output_by_plugin(&self) -> Result<PluginHandle> {
        let mut handle = PluginHandle::default();
        fmod_try!(FMOD_System_GetOutputByPlugin(
            self.as_raw(),
            handle.as_raw_mut(),
        ));
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
        fmod_try!(FMOD_System_CreateDSPByPlugin(
            self.as_raw(),
            handle.into_raw(),
            &mut dsp,
        ));
        Ok(unsafe { Handle::new(dsp) })
    }

    // this functionality needs to wait for custom plugin author binding work.
    // pub fn get_dsp_info_by_plugin(&self, handle: PluginHandle) -> Result<DspDescription> {
    // pub fn register_codec(priority: u32, desc: CodecPluginDescription) -> Result<PluginHandle>;
    // pub fn register_dsp(description: DspDescription) -> Result<PluginHandle>;
    // pub fn register_output(description: OutputDescription) -> Result<PluginHandle>;
}

/// Network configuration.
impl System {
    /// Set a proxy server to use for all subsequent internet connections.
    ///
    /// Specify the proxy in `host:port` format e.g. `www.fmod.com:8888`
    /// (defaults to port 80 if no port is specified).
    ///
    /// Basic authentication is supported using `user:password@host:port` format
    /// e.g. `bob:sekrit123@www.fmod.com:8888`.
    pub fn set_network_proxy(&self, proxy: &CStr) -> Result {
        fmod_try!(FMOD_System_SetNetworkProxy(self.as_raw(), proxy.as_ptr()));
        Ok(())
    }

    /// Retrieves the URL of the proxy server used in internet streaming.
    pub fn get_network_proxy(&self, proxy: &mut String) -> Result {
        unsafe {
            fmod_get_string(proxy, |buf| {
                Error::from_raw(FMOD_System_GetNetworkProxy(
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
        fmod_try!(FMOD_System_SetNetworkTimeout(self.as_raw(), timeout));
        Ok(())
    }

    /// Retrieve the timeout value for network streams.
    pub fn get_network_timeout(&self) -> Result<Duration> {
        let mut timeout = 0;
        fmod_try!(FMOD_System_GetNetworkTimeout(self.as_raw(), &mut timeout));
        Ok(Duration::from_millis(timeout as _))
    }
}

/// A number of playing channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelUsage {
    /// Number of playing [Channel]s (both real and virtual).
    pub all: i32,
    /// Number of playing real (non-virtual) [Channel]s.
    pub real: i32,
}

/// Runnint total information about file reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileUsage {
    /// Total bytes read from file for loading sample data.
    pub sample_bytes_read: i64,
    /// Total bytes read from file for streaming sounds.
    pub stream_bytes_read: i64,
    /// Total bytes read for non-audio data such as FMOD Studio banks.
    pub other_bytes_read: i64,
}

type MixMatrix = [f32; (FMOD_MAX_CHANNEL_WIDTH * FMOD_MAX_CHANNEL_WIDTH) as usize];

/// Information.
impl System {
    /// Retrieves the FMOD version number.
    ///
    /// Compare against `fmod::VERSION` to make sure header and runtime library
    /// versions match.
    pub fn get_version(&self) -> Result<Version> {
        let mut version = 0;
        fmod_try!(FMOD_System_GetVersion(self.as_raw(), &mut version));
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
        fmod_try!(FMOD_System_GetOutputHandle(self.as_raw(), &mut output));
        Ok(output.cast())
    }

    /// Retrieves the number of currently playing Channels.
    ///
    /// For differences between real and virtual voices see the Virtual Voices
    /// guide for more information.
    pub fn get_channels_playing(&self) -> Result<ChannelUsage> {
        let mut channels = 0;
        let mut real_channels = 0;
        fmod_try!(FMOD_System_GetChannelsPlaying(
            self.as_raw(),
            &mut channels,
            &mut real_channels,
        ));
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
        fmod_try!(FMOD_System_GetCPUUsage(self.as_raw(), usage.as_raw_mut()));
        Ok(usage)
    }

    /// Retrieves information about file reads.
    pub fn get_file_usage(&self) -> Result<FileUsage> {
        let mut sample_bytes_read = 0;
        let mut stream_bytes_read = 0;
        let mut other_bytes_read = 0;
        fmod_try!(FMOD_System_GetFileUsage(
            self.as_raw(),
            &mut sample_bytes_read,
            &mut stream_bytes_read,
            &mut other_bytes_read,
        ));
        Ok(FileUsage {
            sample_bytes_read,
            stream_bytes_read,
            other_bytes_read,
        })
    }

    /// Retrieves the default matrix used to convert from one speaker mode to
    /// another.
    ///
    /// The gain for source channel `s` to target channel `t` is
    /// `matrix[t * system.get_speaker_mode_channels(source_mode) + s]`.
    ///
    /// If `source_mode` or `target_mode` is `SpeakerMode::Raw`, this function
    /// will return `Error::InvalidParam`.
    pub fn get_default_mix_matrix(
        &self,
        source_mode: SpeakerMode,
        target_mode: SpeakerMode,
        matrix: &mut MixMatrix,
    ) -> Result {
        fmod_try!(FMOD_System_GetDefaultMixMatrix(
            self.as_raw(),
            source_mode.into_raw(),
            target_mode.into_raw(),
            matrix.as_mut_ptr(),
            0,
        ));
        Ok(())
    }

    /// Retrieves the channel count for a given speaker mode.
    pub fn get_speaker_mode_channels(&self, mode: SpeakerMode) -> Result<usize> {
        let mut channels = 0;
        fmod_try!(FMOD_System_GetSpeakerModeChannels(
            self.as_raw(),
            mode.into_raw(),
            &mut channels,
        ));
        Ok(channels as _)
    }
}

/// Creation and retrieval.
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
    /// Use [Mode::Nonblocking] to have the sound open or load in the
    /// background. You can use [Sound::get_open_state] to determine if it has
    /// finished loading / opening or not. While it is loading (not ready),
    /// sound functions are not accessible for that sound.
    ///
    /// To account for slow media that might cause buffer underrun (skipping /
    /// stuttering / repeating blocks of audio) with sounds created with
    /// [Mode::CreateStream], use [System::set_stream_buffer_size] to increase
    /// read ahead.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    /// Use of Mode::Nonblocking is currently not supported for Wasm.
    /// </pre>
    pub fn create_sound(&self, name: &CStr, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            whoops!(
                trace(
                    ?mode,
                    "System::create_sound called with extended mode; use create_sound_ex instead",
                ),
                panic(
                    "System::create_sound cannot be called with extended mode {mode:?}; use create_sound_ex instead"
                ),
            );
            return Err(Error::InvalidParam);
        }

        let mode = Mode::into_raw(mode);
        let exinfo = ptr::null_mut();
        let mut sound = ptr::null_mut();
        fmod_try!(FMOD_System_CreateSound(
            self.as_raw(),
            name.as_ptr(),
            mode,
            exinfo,
            &mut sound,
        ));
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
    /// Use [Mode::Nonblocking] to have the sound open or load in the
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
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    /// Use of Mode::Nonblocking is currently not supported for Wasm.
    /// </pre>
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
    pub fn create_stream(&self, name: &CStr, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            whoops!(
                trace(
                    ?mode,
                    "System::create_stream called with extended mode; use create_sound_ex instead",
                ),
                panic(
                    "System::create_stream cannot be called with extended mode {mode:?}; use create_sound_ex instead",
                ),
            );
            return Err(Error::InvalidParam);
        }

        let mode = Mode::into_raw(mode);
        let exinfo = ptr::null_mut();
        let mut sound = ptr::null_mut();
        fmod_try!(FMOD_System_CreateStream(
            self.as_raw(),
            name.as_ptr(),
            mode,
            exinfo,
            &mut sound,
        ));
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
        fmod_try!(FMOD_System_CreateDSPByType(
            self.as_raw(),
            kind.into_raw(),
            &mut dsp,
        ));
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
    pub fn create_channel_group(&self, name: &CStr) -> Result<Handle<'_, ChannelGroup>> {
        let mut channel_group = ptr::null_mut();
        fmod_try!(FMOD_System_CreateChannelGroup(
            self.as_raw(),
            name.as_ptr(),
            &mut channel_group,
        ));
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
    pub fn create_sound_group(&self, name: &CStr) -> Result<Handle<'_, SoundGroup>> {
        let mut sound_group = ptr::null_mut();
        fmod_try!(FMOD_System_CreateSoundGroup(
            self.as_raw(),
            name.as_ptr(),
            &mut sound_group,
        ));
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
        fmod_try!(FMOD_System_CreateReverb3D(self.as_raw(), &mut reverb));
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
    ) -> Result<Handle<'_, Channel>> {
        let sound = Sound::as_raw(sound);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        fmod_try!(FMOD_System_PlaySound(
            self.as_raw(),
            sound,
            channelgroup,
            paused as _,
            &mut channel,
        ));
        Ok(unsafe { Handle::new(channel) })
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
    ) -> Result<Handle<'_, Channel>> {
        let dsp = Dsp::as_raw(dsp);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        fmod_try!(FMOD_System_PlayDSP(
            self.as_raw(),
            dsp,
            channelgroup,
            paused as _,
            &mut channel,
        ));
        Ok(unsafe { Handle::new(channel) })
    }

    /// Retrieves a handle to a Channel by ID.
    ///
    /// This function is mainly for getting handles to existing (playing)
    /// [Channel]s and setting their attributes. The only way to 'create' an
    /// instance of a [Channel] for playback is to use [System::play_sound] or
    /// [System::play_dsp].
    pub fn get_channel_count(&self, channel_id: i32) -> Result<Handle<'_, Channel>> {
        let mut channel = ptr::null_mut();
        fmod_try!(FMOD_System_GetChannel(
            self.as_raw(),
            channel_id,
            &mut channel,
        ));
        Ok(unsafe { Handle::new(channel) })
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
        fmod_try!(FMOD_System_GetMasterChannelGroup(
            self.as_raw(),
            &mut channelgroup,
        ));
        Ok(unsafe { ChannelGroup::from_raw(channelgroup) })
    }

    /// Retrieves the default SoundGroup, where all sounds are placed when they
    /// are created.
    ///
    /// If [SoundGroup] is released, the [Sound]s will be put back into this
    /// [SoundGroup].
    pub fn get_master_sound_group(&self) -> Result<&SoundGroup> {
        let mut soundgroup = ptr::null_mut();
        fmod_try!(FMOD_System_GetMasterSoundGroup(
            self.as_raw(),
            &mut soundgroup,
        ));
        Ok(unsafe { SoundGroup::from_raw(soundgroup) })
    }
}

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

/// Runtime control.
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
    /// # let [position_currentframe, position_lastframe, time_taken_since_last_frame_in_seconds] = [0; 3]
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
    ) -> Result<()> {
        fmod_try!(FMOD_System_Set3DListenerAttributes(
            self.as_raw(),
            listener,
            attributes.pos.as_raw(),
            attributes.vel.as_raw(),
            attributes.forward.as_raw(),
            attributes.up.as_raw(),
        ));
        Ok(())
    }

    /// Retrieves the position, velocity and orientation of the specified 3D sound listener.
    ///
    /// Users of the Studio API should call
    /// [studio::System::get_listener_attributes] instead of this function.
    pub fn get_3d_listener_attributes(&self, listener: i32) -> Result<ListenerAttributes3d> {
        let mut attributes = ListenerAttributes3d::default();
        fmod_try!(FMOD_System_Get3DListenerAttributes(
            self.as_raw(),
            listener,
            attributes.pos.as_raw_mut(),
            attributes.vel.as_raw_mut(),
            attributes.forward.as_raw_mut(),
            attributes.up.as_raw_mut(),
        ));
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
    ) -> Result<()> {
        fmod_try!(FMOD_System_SetReverbProperties(
            self.as_raw(),
            instance,
            properties.map_or(ptr::null(), |x| x.as_raw()),
        ));
        Ok(())
    }

    /// Retrieves the current reverb environment for the specified reverb
    /// instance.
    pub fn get_reverb_properties(&self, instance: i32) -> Result<ReverbProperties> {
        let mut properties = ReverbProperties::default();
        fmod_try!(FMOD_System_GetReverbProperties(
            self.as_raw(),
            instance,
            properties.as_raw_mut(),
        ));
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
    ) -> Result<()> {
        fmod_try!(FMOD_System_AttachChannelGroupToPort(
            self.as_raw(),
            port_type.into_raw(),
            port_index.into_raw(),
            group.as_raw(),
            pass_thru as _,
        ));
        Ok(())
    }

    /// Disconnect the output of the specified ChannelGroup from an audio port
    /// on the output driver.
    ///
    /// Removing a [ChannelGroup] from a port will reroute the audio back to the
    /// main mix.
    pub fn detach_channel_group_from_port(&self, channel_group: &ChannelGroup) -> Result<()> {
        fmod_try!(FMOD_System_DetachChannelGroupFromPort(
            self.as_raw(),
            channel_group.as_raw(),
        ));
        Ok(())
    }
}

/// Recording.
impl System {}

/// Geometry management.
impl System {}

/// General.
impl System {}
