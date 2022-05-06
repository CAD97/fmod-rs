use {
    crate::utils::string_extend_utf8_lossy,
    fmod::{raw::*, *},
    parking_lot::RwLockUpgradableReadGuard,
    smart_default::SmartDefault,
    std::{
        borrow::Cow,
        ffi::CStr,
        mem::{self, MaybeUninit},
        os::raw::{c_char, c_int},
        ptr,
    },
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
            if cfg!(debug_assertions) {
                panic!("Only one FMOD system may be created safely. \
                    Read the docs on `System::new_unchecked` if you actually mean to create more than one system. \
                    Note: constructing a studio system automatically creates a core system for you!");
            }

            #[cfg(feature = "tracing")]
            tracing::error!(
                parent: crate::span(),
                "Only one FMOD system may be created safely. \
                    Read the docs on `System::new_unchecked` if you actually mean to create more than one system. \
                    Note: constructing a studio system automatically creates a core system for you!"
            );

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
        name.clear();

        /// the multiple_system.cpp example uses a 256 byte buffer,
        /// so that's probably enough; try that size first
        const DEFAULT_NAME_CAPACITY: usize = 256;

        // only read onto the stack buffer if provided heap buffer isn't larger
        if name.capacity() < DEFAULT_NAME_CAPACITY {
            let mut buffer: [MaybeUninit<u8>; DEFAULT_NAME_CAPACITY] =
                unsafe { MaybeUninit::uninit().assume_init() };

            // first try
            let error = unsafe {
                FMOD_System_GetDriverInfo(
                    self.as_raw(),
                    id,
                    buffer.as_mut_ptr() as *mut c_char,
                    name.capacity() as c_int,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            };

            match Error::from_raw(error) {
                None => {
                    let cstr = unsafe { CStr::from_ptr(buffer.as_ptr() as *const _) };
                    string_extend_utf8_lossy(name, cstr.to_bytes());
                    return Ok(());
                },
                Some(Error::Truncated) => (), // continue
                Some(error) => return Err(error),
            }

            // we'll keep trying with larger buffers I guess
            name.reserve(DEFAULT_NAME_CAPACITY * 2);
        }

        unsafe {
            let name = name.as_mut_vec();
            loop {
                // try again
                let error = FMOD_System_GetDriverInfo(
                    self.as_raw(),
                    id,
                    name.as_mut_ptr() as *mut c_char,
                    name.capacity() as c_int,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                );

                match Error::from_raw(error) {
                    None => break,
                    Some(fmod::Error::Truncated) => {
                        // try doubling the buffer size again?
                        name.reserve(name.len() * 2);
                    },
                    Some(error) => return Err(error),
                }
            }

            // now we need to set the string len and verify it's proper UTF-8
            name.set_len(CStr::from_ptr(name.as_ptr() as *const _).to_bytes().len());
            match String::from_utf8_lossy(name) {
                Cow::Borrowed(_) => {}, // it's valid
                Cow::Owned(fixed) => {
                    // swap in the fixed UTF-8
                    mem::swap(name, &mut fixed.into_bytes());
                },
            }
        }

        Ok(())
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
        pub const fn into_raw(this: Self) -> u32 {
            this.raw
        }
    }
}

/// Plugin support.
impl System {
    pub fn set_plugin_path(&self, path: &CStr) -> Result {
        fmod_try!(FMOD_System_SetPluginPath(self.as_raw(), path.as_ptr()));
        Ok(())
    }

    pub fn load_plugin(&self, filename: &CStr, priority: u32) -> Result<PluginHandle> {
        let mut handle = 0;
        fmod_try!(FMOD_System_LoadPlugin(
            self.as_raw(),
            filename.as_ptr(),
            &mut handle,
            priority,
        ));
        Ok(unsafe { PluginHandle::from_raw(handle) })
    }

    pub fn unload_plugin(&self, handle: PluginHandle) -> Result {
        let handle = PluginHandle::into_raw(handle);
        fmod_try!(FMOD_System_UnloadPlugin(self.as_raw(), handle));
        Ok(())
    }

    pub fn get_num_tested_plugins(&self, handle: PluginHandle) -> Result<i32> {
        let handle = PluginHandle::into_raw(handle);
        let mut count = 0;
        fmod_try!(FMOD_System_GetNumNestedPlugins(
            self.as_raw(),
            handle,
            &mut count,
        ));
        Ok(count)
    }

    pub fn get_nested_plugin(&self, handle: PluginHandle, index: i32) -> Result<PluginHandle> {
        let handle = PluginHandle::into_raw(handle);
        let mut nestedhandle = 0;
        fmod_try!(FMOD_System_GetNestedPlugin(
            self.as_raw(),
            handle,
            index,
            &mut nestedhandle,
        ));
        Ok(unsafe { PluginHandle::from_raw(nestedhandle) })
    }

    // pub fn get_num_plugins(&self, plugin_type: FMOD_PLUGINTYPE) -> Result<i32>;
    // pub fn get_plugin_handle(&self, plugin_type: FMOD_PLUGINTYPE) -> Result<PluginHandle>;
    // pub fn get_plugin_info(&self, handle: PluginHandle) -> (FMOD_PLUGINTYPE, String, u32);

    pub fn set_output_by_plugin(&self, handle: PluginHandle) -> Result {
        let handle = PluginHandle::into_raw(handle);
        fmod_try!(FMOD_System_SetOutputByPlugin(self.as_raw(), handle));
        Ok(())
    }

    pub fn get_output_by_plugin(&self) -> Result<PluginHandle> {
        let mut handle = 0;
        fmod_try!(FMOD_System_GetOutputByPlugin(self.as_raw(), &mut handle));
        Ok(unsafe { PluginHandle::from_raw(handle) })
    }

    pub fn create_dsp_by_plugin(&self, handle: PluginHandle) -> Result<Handle<'_, Dsp>> {
        let handle = PluginHandle::into_raw(handle);
        let mut dsp = ptr::null_mut();
        fmod_try!(FMOD_System_CreateDSPByPlugin(
            self.as_raw(),
            handle,
            &mut dsp,
        ));
        Ok(unsafe { Handle::new(dsp) })
    }

    // pub fn get_dsp_info_by_plugin
    // pub fn register_codec(priority: u32) -> Result<(FMOD_CODEC_DESCRIPTION, PluginHandle)>;
    // pub fn register_dsp(description: FMOD_DSP_DESCRIPTION) -> Result<PluginHandle>;
    // pub fn register_output(description: FMOD_OUTPUT_DESCRIPTION) -> Result<PluginHandle>;
}

/// Network configuration.
impl System {
    // pub fn set_network_proxy
    // pub fn get_network_proxy
    // pub fn set_network_timeout
    // pub fn get_network_timeout
}

/// Information.
impl System {
    // pub fn get_version
    // pub fn get_output_handle

    pub fn get_channels_playing(&self) -> Result<(i32, i32)> {
        let mut channels = 0;
        let mut real_channels = 0;
        fmod_try!(FMOD_System_GetChannelsPlaying(
            self.as_raw(),
            &mut channels,
            &mut real_channels,
        ));
        Ok((channels, real_channels))
    }

    // pub fn get_cpu_usage
    // pub fn get_file_usage
    // pub fn get_default_mix_matrix
    // pub fn get_speaker_mode_channels
}

/// Creation and retrieval.
impl System {
    // TODO: create_sound_ex
    pub fn create_sound(&self, name: &CStr, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            if cfg!(debug_assertions) {
                panic!("System::create_sound cannot be called with extended mode {mode:?}; use create_sound_ex instead");
            }
            #[cfg(feature = "tracing")]
            tracing::error!(
                parent: crate::span(),
                ?mode,
                "System::create_sound called with extended mode; use create_sound_ex instead",
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

    // TODO: create_stream_ex
    pub fn create_stream(&self, name: &CStr, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            if cfg!(debug_assertions) {
                panic!("System::create_stream cannot be called with extended mode {mode:?}; use create_stream_ex instead");
            }
            #[cfg(feature = "tracing")]
            tracing::error!(
                parent: crate::span(),
                ?mode,
                "System::create_stream called with extended mode; use create_stream_ex instead",
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

    // pub fn create_dsp
    // pub fn create_dsp_by_type

    pub fn create_channel_group(&self, name: &CStr) -> Result<Handle<'_, ChannelGroup>> {
        let mut channel_group = ptr::null_mut();
        fmod_try!(FMOD_System_CreateChannelGroup(
            self.as_raw(),
            name.as_ptr(),
            &mut channel_group,
        ));
        Ok(unsafe { Handle::new(channel_group) })
    }

    // pub fn create_sound_group
    // pub fn create_reverb_3d

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

    // pub fn play_dsp
    // pub fn get_channel
    // pub fn get_dsp_info_by_type
    // pub fn get_master_channel_group
    // pub fn get_master_sound_group
}

/// Runtime control.
impl System {}

/// Recording.
impl System {}

/// Geometry management.
impl System {}

/// General.
impl System {}
