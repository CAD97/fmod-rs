use {
    crate::{
        fmod_debug_install_tracing, raw::*, Channel, ChannelGroup, Dsp, Error, FmodResource,
        Handle, InitFlags, Mode, Result, Sound,
    },
    std::{ffi::CString, path::Path, ptr},
};

opaque! {
    class System;
}

impl System {
    /// Creates an instance of the FMOD system.
    ///
    /// # FMOD.rs implementation note
    ///
    /// FMOD.rs internally maintains that only a single FMOD System is created.
    /// This ensures that FMOD System creation and destruction, which are thread
    /// unsafe, are unable to race with any other FMOD functionality. The System
    /// is only released once the final `Handle` (to any FMOD type, not just the
    /// System itself) is dropped (the System is reference counted).
    pub fn new() -> Result<Handle<Self>> {
        Handle::new_system()
    }

    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut FMOD_SYSTEM) -> &'static System {
            &*(raw as *mut _)
        }

        pub fn as_raw(&self) -> *mut FMOD_SYSTEM {
            self as *const _ as *const _ as *mut _
        }
    }

    pub(super) unsafe fn raw_create() -> Result<*mut Self> {
        // log setup
        #[cfg(feature = "fmod_debug_is_tracing")]
        fmod_debug_install_tracing()?;

        // actual creation
        let mut raw = ptr::null_mut();
        let result = FMOD_System_Create(&mut raw, FMOD_VERSION);
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(raw as *mut Self)
        }
    }

    pub(super) unsafe fn raw_release(this: *mut Self) -> Result<()> {
        let result = FMOD_System_Release(this as *mut _);
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }
}

unsafe impl FmodResource for System {
    type Raw = FMOD_SYSTEM;

    unsafe fn release(_this: *mut FMOD_SYSTEM) -> Result<()> {
        // System is released by the Handle's beating heart
        Ok(())
    }
}

/// Setup functions.
impl System {
    // pub fn set_output(&self, output: FMOD_OUTPUTTYPE) -> Result;
    // pub fn get_output(&self) -> Result<FMOD_OUTPUTTYPE>;

    pub fn get_num_drivers(&self) -> Result<i32> {
        let mut numdrivers = 0;
        let result = unsafe { FMOD_System_GetNumDrivers(self.as_raw(), &mut numdrivers) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(numdrivers)
        }
    }

    // pub fn get_driver_info(&self, id: i32) -> Result<(String, FMOD_GUID, i32, FMOD_SPEAKERMODE, i32)>;

    pub fn set_driver(&self, id: i32) -> Result {
        let result = unsafe { FMOD_System_SetDriver(self.as_raw(), id) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn get_driver(&self) -> Result<i32> {
        let mut driver = 0;
        let result = unsafe { FMOD_System_GetDriver(self.as_raw(), &mut driver) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(driver)
        }
    }

    pub fn set_software_channels(&self, num_software_channels: i32) -> Result {
        let result =
            unsafe { FMOD_System_SetSoftwareChannels(self.as_raw(), num_software_channels) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    // pub fn set_software_format(&self, sample_rate: i32, speaker_mode: FMOD_SPEAKERMODE, num_raw_speakers: i32) -> Result;
    // pub fn get_software_format(&self) -> Result<(i32, FMOD_SPEAKERMODE, i32)>;

    pub fn set_dsp_buffer_size(&self, buffer_length: u32, num_buffers: i32) -> Result {
        let result =
            unsafe { FMOD_System_SetDSPBufferSize(self.as_raw(), buffer_length, num_buffers) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn get_dsp_buffer_size(&self) -> Result<(u32, i32)> {
        let mut bufferlength = 0;
        let mut numbuffers = 0;
        let result = unsafe {
            FMOD_System_GetDSPBufferSize(self.as_raw(), &mut bufferlength, &mut numbuffers)
        };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok((bufferlength, numbuffers))
        }
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
    pub fn set_plugin_path(&self, path: impl AsRef<Path>) -> Result {
        let path = path.as_ref().to_str().ok_or(Error::FileBad)?;
        let path = CString::new(path).map_err(|_| Error::FileBad)?;
        let result = unsafe { FMOD_System_SetPluginPath(self.as_raw(), path.as_ptr()) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn load_plugin(&self, filename: impl AsRef<Path>, priority: u32) -> Result<PluginHandle> {
        let filename = filename.as_ref().to_str().ok_or(Error::FileBad)?;
        let filename = CString::new(filename).map_err(|_| Error::FileBad)?;
        let mut handle = 0;
        let result = unsafe {
            FMOD_System_LoadPlugin(self.as_raw(), filename.as_ptr(), &mut handle, priority)
        };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(unsafe { PluginHandle::from_raw(handle) })
        }
    }

    pub fn unload_plugin(&self, handle: PluginHandle) -> Result {
        let handle = PluginHandle::into_raw(handle);
        let result = unsafe { FMOD_System_UnloadPlugin(self.as_raw(), handle) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn get_num_tested_plugins(&self, handle: PluginHandle) -> Result<i32> {
        let handle = PluginHandle::into_raw(handle);
        let mut count = 0;
        let result = unsafe { FMOD_System_GetNumNestedPlugins(self.as_raw(), handle, &mut count) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(count)
        }
    }

    pub fn get_nested_plugin(&self, handle: PluginHandle, index: i32) -> Result<PluginHandle> {
        let handle = PluginHandle::into_raw(handle);
        let mut nestedhandle = 0;
        let result =
            unsafe { FMOD_System_GetNestedPlugin(self.as_raw(), handle, index, &mut nestedhandle) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(unsafe { PluginHandle::from_raw(nestedhandle) })
        }
    }

    // pub fn get_num_plugins(&self, plugin_type: FMOD_PLUGINTYPE) -> Result<i32>;
    // pub fn get_plugin_handle(&self, plugin_type: FMOD_PLUGINTYPE) -> Result<PluginHandle>;
    // pub fn get_plugin_info(&self, handle: PluginHandle) -> (FMOD_PLUGINTYPE, String, u32);

    pub fn set_output_by_plugin(&self, handle: PluginHandle) -> Result {
        let handle = PluginHandle::into_raw(handle);
        let result = unsafe { FMOD_System_SetOutputByPlugin(self.as_raw(), handle) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn get_output_by_plugin(&self) -> Result<PluginHandle> {
        let mut handle = 0;
        let result = unsafe { FMOD_System_GetOutputByPlugin(self.as_raw(), &mut handle) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(unsafe { PluginHandle::from_raw(handle) })
        }
    }

    pub fn create_dsp_by_plugin(&self, handle: PluginHandle) -> Result<Handle<Dsp>> {
        let handle = PluginHandle::into_raw(handle);
        let mut dsp = ptr::null_mut();
        let result = unsafe { FMOD_System_CreateDSPByPlugin(self.as_raw(), handle, &mut dsp) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            unsafe { Handle::new_raw(dsp) }
        }
    }

    // pub fn register_codec(priority: u32) -> Result<(FMOD_CODEC_DESCRIPTION, PluginHandle)>;
    // pub fn register_codec(description: FMOD_DSP_DESCRIPTION) -> Result<PluginHandle>;
    // pub fn register_codec(description: FMOD_OUTPUT_DESCRIPTION) -> Result<PluginHandle>;
}

opaque! {
    class ExtraDriverData;
}

/// Init/Close.
impl System {
    pub fn init(&self, max_channels: i32, flags: InitFlags) -> Result {
        let flags = InitFlags::into_raw(flags);
        // TODO: figure out what this parameter is for
        let extradriverdata = ptr::null_mut();
        let result =
            unsafe { FMOD_System_Init(self.as_raw(), max_channels, flags, extradriverdata) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn close(&self) -> Result {
        let result = unsafe { FMOD_System_Close(self.as_raw()) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }
}

/// General post-init system functions.
impl System {
    pub fn update(&self) -> Result {
        let result = unsafe { FMOD_System_Update(self.as_raw()) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }

    // snip
}

/// System information functions.
impl System {
    // snip

    pub fn get_channels_playing(&self) -> Result<i32> {
        let mut channels = 0;
        // TODO: expose this optional return value
        let realchannels = ptr::null_mut();
        let result =
            unsafe { FMOD_System_GetChannelsPlaying(self.as_raw(), &mut channels, realchannels) };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(channels)
        }
    }

    // snip
}

/// Sound/DSP/Channel/FX creation and retrieval.
impl System {
    pub fn create_sound(&self, name: impl AsRef<Path>, mode: Mode) -> Result<Handle<Sound>> {
        // TODO: ??name_or_data??
        // TODO: expose exinfo
        let name_or_data = name.as_ref().to_str().ok_or(Error::FileBad)?;
        let name_or_data = CString::new(name_or_data).map_err(|_| Error::FileBad)?;
        let mode = Mode::into_raw(mode);
        let mut sound = ptr::null_mut();
        // TODO(SAFETY): make sure that the name_or_data lifetime is okay
        let result = unsafe {
            FMOD_System_CreateSound(
                self.as_raw(),
                name_or_data.as_ptr(),
                mode,
                ptr::null_mut(),
                &mut sound,
            )
        };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            unsafe { Handle::new_raw(sound) }
        }
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
        let result = unsafe {
            FMOD_System_PlaySound(
                self.as_raw(),
                sound,
                channelgroup,
                paused as _,
                &mut channel,
            )
        };
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            debug_assert!(!channel.is_null());
            unsafe { Handle::new_raw(channel) }
        }
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
