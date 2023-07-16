use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    std::ptr,
};

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
