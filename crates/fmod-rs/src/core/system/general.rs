use {
    crate::utils::catch_user_unwind,
    fmod::{raw::*, *},
    std::{
        borrow::Cow,
        ffi::{c_char, c_void, CStr},
        marker::PhantomData,
        mem::ManuallyDrop,
    },
};

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
    /// by calling [`System::unlock_dsp`].
    ///
    /// Note that the DSP engine should not be locked for a significant amount
    /// of time, otherwise inconsistency in the audio output may result.
    /// (audio skipping / stuttering).
    ///
    /// # Safety
    ///
    /// The DSP engine must not already be locked when this function is called.
    pub unsafe fn lock_dsp(&self) -> Result {
        ffi!(FMOD_System_LockDSP(self.as_raw()))?;
        Ok(())
    }

    /// Mutual exclusion function to unlock the FMOD DSP engine (which runs
    /// asynchronously in another thread) and let it continue executing.
    ///
    /// # Safety
    ///
    /// The DSP engine must be locked with [`System::lock_dsp`] when this
    /// function is called.
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
        unsafe { Instance::Unknown(&*self.instance.cast()) }
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

/// Callbacks called by the [`System`].
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
    fn thread_created(system: &System, thread: SystemThreadHandle, name: &str) -> Result {
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
    fn thread_destroyed(system: &System, thread: SystemThreadHandle, name: &str) -> Result {
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
            let thread = commanddata1 as SystemThreadHandle;
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
            let thread = commanddata1 as SystemThreadHandle;
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

/// An instance of some FMOD object. Passed to error callbacks.
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum Instance<'a> {
    #[doc(hidden)]
    Unknown(&'a ()),
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

impl Instance<'_> {
    pub fn addr(self) -> usize {
        match self {
            Instance::Unknown(p) => p as *const _ as _,
            Instance::System(p) => p as *const _ as _,
            Instance::Channel(p) => p as *const _ as _,
            Instance::ChannelGroup(p) => p as *const _ as _,
            Instance::ChannelControl(p) => p as *const _ as _,
            Instance::Sound(p) => p as *const _ as _,
            Instance::SoundGroup(p) => p as *const _ as _,
            Instance::Dsp(p) => p as *const _ as _,
            Instance::DspConnection(p) => p as *const _ as _,
            Instance::Geometry(p) => p as *const _ as _,
            Instance::Reverb3d(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioSystem(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioEventDescription(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioEventInstance(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioBus(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioVca(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioBank(p) => p as *const _ as _,
            // #[cfg(feature = "studio")]
            // Instance::StudioCommandReplay(p) => p as *const _ as _,
        }
    }
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

/// Mutual exclusion lock guard for the FMOD DSP engine.
///
/// The lock is released when this guard is dropped.
pub struct DspLock<'a> {
    system: &'a System,
}

impl DspLock<'_> {
    /// Mutual exclusion function to lock the FMOD DSP engine (which runs
    /// asynchronously in another thread), so that it will not execute.
    ///
    /// See [`System::lock_dsp`] for more information.
    ///
    /// # Safety
    ///
    /// The DSP engine must not already be locked when this function is called.
    pub unsafe fn new(system: &System) -> Result<DspLock<'_>> {
        system.lock_dsp()?;
        Ok(DspLock { system })
    }

    /// Mutual exclusion function to unlock the FMOD DSP engine (which runs
    /// asynchronously in another thread) and let it continue executing.
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
                whoops!("error unlocking DSP engine: {e}");
            },
        }
    }
}

#[cfg(all(not(doc), windows))]
pub type SystemThreadHandle = std::os::windows::io::RawHandle;
#[cfg(all(not(doc), unix))]
pub type SystemThreadHandle = std::os::unix::thread::RawPthread;

/// A raw OS thread handle.
///
/// - Unix: [`std::os::unix::thread::RawPthread`]
/// - Windows: [`std::os::windows::io::RawHandle`]
#[cfg(doc)]
#[cfg_attr(feature = "unstable", doc(cfg(any(unix, windows))))]
pub type SystemThreadHandle = unknown::SystemThreadHandle;

#[cfg(doc)]
mod unknown {
    pub struct SystemThreadHandle(*mut ());
}
