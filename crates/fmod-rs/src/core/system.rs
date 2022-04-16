use {
    fmod::{
        raw::*, Channel, ChannelGroup, Dsp, Error, Guid, Handle, InitFlags, Mode, OutputType,
        Result, Sound, SpeakerMode, GLOBAL_SYSTEM_STATE,
    },
    std::{
        borrow::Cow,
        ffi::CStr,
        mem,
        os::raw::{c_char, c_int},
        ptr,
        sync::atomic::Ordering,
    },
};

opaque!(class System = FMOD_SYSTEM, FMOD_System_*);

impl System {
    /// Create an instance of the FMOD system.
    ///
    /// Only a single system
    #[cfg_attr(
        feature = "studio",
        doc = " (or [studio system][fmod::studio::System])"
    )]
    /// can be constructed with this function; further attempts to create a
    /// system will return an error. See [`new_unchecked`][Self::new_unchecked]
    /// for more information about why having multiple systems is unsafe.
    pub fn new() -> Result<&'static Self> {
        // guard against multiple system creation racing
        if GLOBAL_SYSTEM_STATE
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
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

        // log setup
        crate::fmod_debug_install_tracing();

        // actual creation
        unsafe { Self::new_unchecked() }
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
    /// stale/misused handles can occur. If you want to release a system, you
    /// can use [`Handle::unleak`], but need to ensure that dropping the handle
    /// cannot race with any other FMOD API calls (and, of course, that all
    /// resources in that system are cleaned up).
    ///
    /// # Safety
    ///
    /// In summary, if you construct multiple systems, you must:
    ///
    /// - Ensure that system creation and releasing does not potentially race
    ///   any FMOD API calls.
    /// - Ensure that handles created in one system are not used with a
    ///   different system.
    pub unsafe fn new_unchecked() -> Result<&'static Self> {
        let mut raw = ptr::null_mut();
        fmod_try!(FMOD_System_Create(&mut raw, FMOD_VERSION));
        GLOBAL_SYSTEM_STATE.store(2, Ordering::Release);
        Ok(Handle::leak(Handle::new(raw)))
    }
}

/// Identification information about a sound device specified by its index,
/// specific to the selected output mode.
#[derive(Debug, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq, Eq, Hash)]
/// Output format for the software mixer
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

/// Setup functions.
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
    // requirements to validate non-truncation and UTF-8.

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
        /// The multiple_system.cpp example uses a 256 byte buffer,
        /// so that's probably enough; try that size first
        const DEFAULT_NAME_CAPACITY: usize = 256;

        name.clear();
        unsafe {
            let name = name.as_mut_vec();
            let mut error = Some(fmod::Error::Truncated);
            while let Some(terror) = error {
                match terror {
                    fmod::Error::Truncated => {
                        if name.capacity() < DEFAULT_NAME_CAPACITY {
                            name.reserve(DEFAULT_NAME_CAPACITY);
                        } else {
                            // try doubling the buffer size?
                            name.reserve(name.len() * 2);
                        }
                    },
                    error => return Err(error)?,
                }

                // try again
                error = fmod::Error::from_raw(FMOD_System_GetDriverInfo(
                    self.as_raw(),
                    id,
                    name.as_mut_ptr() as *mut c_char,
                    name.capacity() as c_int,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                ));
            }

            // now we need to set the string len and verify it's UTF-8
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
            &mut num_software_channels
        ));
        Ok(num_software_channels)
    }

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
            num_raw_speakers
        ));
        Ok(())
    }

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

    pub fn set_dsp_buffer_size(&self, buffer_length: u32, num_buffers: i32) -> Result {
        fmod_try!(FMOD_System_SetDSPBufferSize(
            self.as_raw(),
            buffer_length,
            num_buffers,
        ));
        Ok(())
    }

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

    // pub fn set_file_system(
    //     &self,
    //     user_open: FMOD_FILE_OPEN_CALLBACK,
    //     user_close: FMOD_FILE_CLOSE_CALLBACK,
    //     user_read: FMOD_FILE_READ_CALLBACK,
    //     user_seek: FMOD_FILE_SEEK_CALLBACK,
    //     user_async_read: FMOD_FILE_ASYNCREAD_CALLBACK,
    //     user_async_cancel: FMOD_FILE_ASYNCCANCEL_CALLBACK,
    //     block_align: i32,
    // ) -> Result {
    // }

    // pub fn attach_file_system(
    //     &self,
    //     user_open: FMOD_FILE_OPEN_CALLBACK,
    //     user_close: FMOD_FILE_CLOSE_CALLBACK,
    //     user_read: FMOD_FILE_READ_CALLBACK,
    //     user_seek: FMOD_FILE_SEEK_CALLBACK,
    // ) -> Result {
    // }

    // pub fn set_advanced_settings(&self, settings: FMOD_ADVANCEDSETTINGS) -> Result;
    // pub fn get_advanced_settings(&self) -> Result<FMOD_ADVANCEDSETTINGS>;

    // pub fn set_callback(
    //     callback: FMOD_SYSTEM_CALLBACK,
    //     callbackmask: FMOD_SYSTEM_CALLBACK_TYPE,
    // ) -> Result {
    // }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Plug-in support.
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

    pub fn create_dsp_by_plugin(&self, handle: PluginHandle) -> Result<Handle<Dsp>> {
        let handle = PluginHandle::into_raw(handle);
        let mut dsp = ptr::null_mut();
        fmod_try!(FMOD_System_CreateDSPByPlugin(
            self.as_raw(),
            handle,
            &mut dsp,
        ));
        Ok(unsafe { Handle::new(dsp) })
    }

    // pub fn register_codec(priority: u32) -> Result<(FMOD_CODEC_DESCRIPTION, PluginHandle)>;
    // pub fn register_codec(description: FMOD_DSP_DESCRIPTION) -> Result<PluginHandle>;
    // pub fn register_codec(description: FMOD_OUTPUT_DESCRIPTION) -> Result<PluginHandle>;
}

/// Init/Close.
impl System {
    /// Initialize the system object and prepare FMOD for playback.
    ///
    /// Most API functions require an initialized System object before they will succeed, otherwise they will return [Error::Uninitialized]. Some can only be called before initialization. These are:
    ///
    /// - [memory_initialize]
    /// - [System::set_software_format]
    /// - [System::set_software_channels]
    /// - [System::set_dsp_buffer_size]
    ///
    /// [System::set_output] / [System::set_output_by_plugin] can be called before or after [System::init] on Android, GameCore, UWP, Windows and Mac. Other platforms can only call this **before** [System::init].
    ///
    /// ## max_channels
    ///
    /// Maximum number of [Channel] objects available for playback, also known as virtual channels. Virtual channels will play with minimal overhead, with a subset of 'real' voices that are mixed, and selected based on priority and audibility. See the Virtual Voices guide for more information.
    /// Range: [0, 4095]
    ///
    /// ## flags
    ///
    /// Initialization flags. More than one mode can be set at once by combining them with the OR operator.
    pub fn init(&self, max_channels: i32, flags: InitFlags) -> Result {
        // I hope FMOD does the right thing for a nullptr driver data in all cases...
        unsafe { self.init_ex(max_channels, flags, ptr::null()) }
    }

    /// Initialize the system object and prepare FMOD for playback.
    ///
    /// # Safety
    ///
    /// `extra_driver_data` must be correct.
    ///
    /// ## extra_driver_data
    ///
    /// Additional output specific initialization data. This will be passed to the output plugin. See [OutputType] for descriptions of data that can be passed in, based on the selected output mode.
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
            extra_driver_data as *mut _
        ));
        Ok(())
    }

    // TODO: safe init_ex wrappers for WavWriter[Nrt], PulseAudio
}

/// General post-init system functions.
impl System {
    pub fn update(&self) -> Result {
        fmod_try!(FMOD_System_Update(self.as_raw()));
        Ok(())
    }

    // snip
}

/// System information functions.
impl System {
    // snip

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

    // snip
}

/// Sound/DSP/Channel/FX creation and retrieval.
impl System {
    // TODO: create_cound_ex
    pub fn create_sound(&self, name: &CStr, mode: Mode) -> Result<Handle<Sound>> {
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

    // snip

    pub fn create_channel_group(&self, name: &CStr) -> Result<Handle<ChannelGroup>> {
        let mut channel_group = ptr::null_mut();
        fmod_try!(FMOD_System_CreateChannelGroup(
            self.as_raw(),
            name.as_ptr(),
            &mut channel_group,
        ));
        Ok(unsafe { Handle::new(channel_group) })
    }

    // snip

    pub fn play_sound(
        &self,
        sound: &Sound,
        channel_group: Option<&ChannelGroup>,
        paused: bool,
    ) -> Result<Handle<Channel>> {
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

    // snip
}

/// Routing to ports.
impl System {
    // snip
}

/// Reverb API.
impl System {
    // snip
}

/// System level DSP functionality.
impl System {
    // snip
}

/// Recording API.
impl System {
    // snip
}

/// Geometry API.
impl System {
    // snip
}

/// Network functions.
impl System {
    // snip
}

/// Userdata set/get.
impl System {
    // snip
}
