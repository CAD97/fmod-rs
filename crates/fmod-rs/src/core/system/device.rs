use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    smart_default::SmartDefault,
    std::ptr,
};

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
    pub fn set_output(&self, output: OutputType) -> Result {
        let output = output.into_raw();
        ffi!(FMOD_System_SetOutput(self.as_raw(), output))?;
        Ok(())
    }

    /// Retrieves the type of output interface used to run the mixer.
    pub fn get_output(&self) -> Result<OutputType> {
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
            state: DriverState::zeroed(),
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

fmod_enum! {
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

/// Identification information about a sound device.
#[derive(Debug, SmartDefault, Copy, Clone, PartialEq, Eq, Hash)]
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
    /// Only meaningful for record drivers.
    #[default(DriverState::zeroed())]
    pub state: DriverState,
}
