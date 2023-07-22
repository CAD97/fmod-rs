use {
    fmod::{raw::*, *},
    std::{mem::ManuallyDrop, ptr, slice},
};

/// # Data reading.
impl Sound {
    /// Retrieves the state a sound is in after being opened with the non
    /// blocking flag, or the current state of the streaming buffer.
    ///
    /// When a sound is opened with [`Mode::NonBlocking`], it is opened and
    /// prepared in the background, or asynchronously. This allows the main
    /// application to execute without stalling on audio loads. This function
    /// will describe the state of the asynchronous load routine i.e. whether
    /// it has succeeded, failed or is still in progress.
    ///
    /// **Note:** Always check the return value to determine the state of the
    /// sound. Do not assume that if this function returns `Ok` then the sound
    /// has finished loading.
    pub fn get_open_state(&self) -> Result<OpenState> {
        let mut state = OpenState::zeroed();
        ffi!(FMOD_Sound_GetOpenState(
            self.as_raw(),
            state.as_raw_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        ))?;
        Ok(state)
    }

    /// Retrieves the state a sound is in after being opened with the non
    /// blocking flag, or the current state of the streaming buffer.
    ///
    /// When a sound is opened with [`Mode::NonBlocking`], it is opened and
    /// prepared in the background, or asynchronously. This allows the main
    /// application to execute without stalling on audio loads. This function
    /// will describe the state of the asynchronous load routine i.e. whether
    /// it has succeeded, failed or is still in progress.
    ///
    /// If `starving` is true, then you will most likely hear a
    /// stuttering/repeating sound as the decode buffer loops on itself and
    /// replays old data. With the ability to detect stream starvation, muting
    /// the sound with [`ChannelControl::set_mute`] will keep the stream quiet
    /// until it is not starving any more.
    ///
    /// **Note:** Always check the return value to determine the state of the
    /// sound. Do not assume that if this function returns `Ok` then the sound
    /// has finished loading.
    pub fn get_open_state_info(&self) -> Result<OpenStateInfo> {
        let mut state = OpenState::zeroed();
        let mut percent_buffered = 0;
        let mut starving = 0;
        let mut disk_busy = 0;
        ffi!(FMOD_Sound_GetOpenState(
            self.as_raw(),
            state.as_raw_mut(),
            &mut percent_buffered,
            &mut starving,
            &mut disk_busy
        ))?;
        Ok(OpenStateInfo {
            state,
            percent_buffered,
            starving: starving != 0,
            disk_busy: disk_busy != 0,
        })
    }

    /// Reads data from an opened sound to a specified buffer,
    /// using FMOD's internal codecs.
    ///
    /// This can be used for decoding data offline in small pieces (or big
    /// pieces), rather than playing and capturing it, or loading the whole file
    /// at once and having to [`Sound::lock`] / [`Sound::unlock`] the data.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">🦀</span><span>
    /// FMOD.rs returns `Ok(0)` on EOF, matching the [`Read`](io::Read) trait,
    /// whereas raw FMOD returns `Error::FileEof`.
    /// </span></div></div>
    ///
    /// As a non streaming sound reads and decodes the whole file then closes it
    /// upon calling [`System::create_sound`], [`Sound::read_data`] will then
    /// not work because the file handle is closed. Use [`Mode::OpenOnly`] to
    /// stop FMOD reading/decoding the file. If [`Mode::OpenOnly`] flag is used
    /// when opening a sound, it will leave the file handle open, and FMOD will
    /// not read/decode any data internally, so the read cursor will stay at
    /// position 0. This will allow the user to read the data from the start.
    ///
    /// For streams, the streaming engine will decode a small chunk of data and
    /// this will advance the read cursor. You need to either use
    /// [`Mode::OpenOnly`] to stop the stream pre-buffering or call
    /// [`Sound::seek_data`] to reset the read cursor back to the start of the
    /// file, otherwise it will appear as if the start of the stream is missing.
    /// [`Channel::set_position`] will have the same result. These functions
    /// will flush the stream buffer and read in a chunk of audio internally.
    /// This is why if you want to read from an absolute position you should use
    /// [`Sound::seek_data`] and not the previously mentioned functions.
    ///
    /// If you are calling [`Sound::read_data`] and [`Sound::seek_data`] on a
    /// stream, information functions such as [`Channel::get_position`] may give
    /// misleading results. Calling [`Channel::set_position`] will cause the
    /// streaming engine to reset and flush the stream, leading to the time
    /// values returning to their correct position.
    pub fn read_data(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut read = 0;
        ffi!(FMOD_Sound_ReadData(
            self.as_raw(),
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
            &mut read,
        ))
        .or_else(|e| if e == Error::FileEof { Ok(()) } else { Err(e) })?;
        Ok(ix!(read))
    }

    /// Seeks a sound for use with data reading, using FMOD's internal codecs.
    ///
    /// For use in conjunction with [`Sound::read_data`] and [`Mode::OpenOnly`].
    ///
    /// For streaming sounds, if this function is called, it will advance the
    /// internal file pointer but not update the streaming engine. This can lead
    /// to de-synchronization of position information for the stream and audible
    /// playback.
    ///
    /// A stream can reset its stream buffer and position synchronization by
    /// calling [`Channel::set_position`]. This causes reset and flush of the
    /// stream buffer.
    pub fn seek_data(&self, pcm: u32) -> Result {
        ffi!(FMOD_Sound_SeekData(self.as_raw(), pcm))?;
        Ok(())
    }

    /// Gives access to a portion or all the sample data of a sound for direct manipulation.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">🦀</span><span>
    /// Dropping `SoundReadGuard` will call `Sound::unlock` and unlock the data.
    /// </span></div></div>
    ///
    /// With this function you get access to the raw audio data. If the data is
    /// 8, 16, 24 or 32bit PCM data, mono or stereo data, you must take this
    /// into consideration when processing the data. See [Sample Data] for more
    /// information.
    ///
    /// [Sample Data]: https://fmod.com/docs/2.02/api/glossary.html#sample-data
    ///
    /// If the sound is created with [`Mode::CreateCompressedSample`] the data
    /// retrieved will be the compressed bitstream.
    ///
    /// It is not possible to lock the following:
    ///
    /// - A parent sound containing subsounds. A parent sound has no audio data
    ///   and [`Error::SubSounds`] will be returned.
    /// - A stream / sound created with [`Mode::CreateStream`].
    ///   [`Error::BadCommand`] will be returned in this case.
    ///
    /// The names `lock`/`unlock` are a legacy reference to older Operating
    /// System APIs that used to cause a mutex lock on the data, so that it
    /// could not be written to while the 'lock' was in place. This is no
    /// longer the case with FMOD and data can be 'locked' multiple times
    /// from different places/threads at once.
    pub fn lock(&self, offset: u32, length: u32) -> Result<SampleDataLock<'_>> {
        let mut ptr1 = ptr::null_mut();
        let mut ptr2 = ptr::null_mut();
        let mut len1 = 0;
        let mut len2 = 0;
        ffi!(FMOD_Sound_Lock(
            self.as_raw(),
            offset,
            length,
            &mut ptr1,
            &mut ptr2,
            &mut len1,
            &mut len2,
        ))?;
        unsafe {
            Ok(SampleDataLock {
                sound: self,
                part1: slice::from_raw_parts(ptr1.cast(), ix!(len1)),
                part2: if !ptr2.is_null() {
                    slice::from_raw_parts(ptr2.cast(), ix!(len2))
                } else {
                    slice::from_raw_parts(ptr1.cast::<u8>().add(ix!(len1)), 0)
                },
            })
        }
    }

    /// Finalizes a previous sample data lock and submits it back to the Sound
    /// object.
    ///
    /// If an unlock is not performed on PCM data, then sample loops may produce
    /// audible clicks.
    ///
    /// The names `lock`/`unlock` are a legacy reference to older Operating
    /// System APIs that used to cause a mutex lock on the data, so that it
    /// could not be written to while the 'lock' was in place. This is no
    /// longer the case with FMOD and data can be 'locked' multiple times
    /// from different places/threads at once.
    ///
    /// # Safety
    ///
    /// The locked slices must have been obtained from a previous matched call
    /// to [`Sound::lock`].
    pub unsafe fn unlock(&self, part1: &[u8], part2: &[u8]) -> Result {
        ffi!(FMOD_Sound_Unlock(
            self.as_raw(),
            part1.as_ptr().cast_mut().cast(),
            part2.as_ptr().cast_mut().cast(),
            part1.len() as u32,
            part2.len() as u32,
        ))?;
        Ok(())
    }
}

fmod_enum! {
    /// These values describe what state a sound is in after [Mode::NonBlocking] has been used to open it.
    ///
    /// With streams, if you are using [Mode::NonBlocking], note that if the user calls [Sound::get_sub_sound], a stream will go into [OpenState::Seeking] state and sound related commands will return [Error::NotReady].
    ///
    /// With streams, if you are using [Mode::NonBlocking], note that if the user calls [Channel::get_position], a stream will go into [OpenState::SetPosition] state and sound related commands will return [Error::NotReady].
    pub enum OpenState: FMOD_OPENSTATE {
        /// Opened and ready to play.
        Ready       = FMOD_OPENSTATE_READY,
        /// Initial load in progress.
        Loading     = FMOD_OPENSTATE_LOADING,
        /// Failed to open - file not found, out of memory etc. See return value of [Sound::get_open_state] for what happened.
        Error       = FMOD_OPENSTATE_ERROR,
        /// Connecting to remote host (internet sounds only).
        Connecting  = FMOD_OPENSTATE_CONNECTING,
        /// Buffering data.
        Buffering   = FMOD_OPENSTATE_BUFFERING,
        /// Seeking to subsound and re-flushing stream buffer.
        Seeking     = FMOD_OPENSTATE_SEEKING,
        /// Ready and playing, but not possible to release at this time without stalling the main thread.
        Playing     = FMOD_OPENSTATE_PLAYING,
        /// Seeking within a stream to a different position.
        SetPosition = FMOD_OPENSTATE_SETPOSITION,
    }
}

// XXX: io::Read and io::Seek impls?

/// A read lock on a sound's sample data.
#[derive(Debug, Clone)]
pub struct SampleDataLock<'a> {
    sound: &'a Sound,
    part1: &'a [u8],
    part2: &'a [u8],
}

impl SampleDataLock<'_> {
    /// Returns the locked sample data.
    ///
    /// The first slice borrows from the sample buffer directly. If the locked
    /// data exceeds the length of the sample buffer, the second slice holds
    /// any excess data.
    pub fn get(&self) -> (&[u8], &[u8]) {
        (self.part1, self.part2)
    }

    /// Finalizes the sample data lock and submits it back to the Sound object.
    pub fn unlock(self) -> Result {
        let this = ManuallyDrop::new(self);
        unsafe { this.sound.unlock(this.part1, this.part2) }
    }
}

impl<'a> Drop for SampleDataLock<'a> {
    fn drop(&mut self) {
        match unsafe { self.sound.unlock(self.part1, self.part2) } {
            Ok(()) => (),
            Err(e) => whoops!("failed to unlock sound: {e}"),
        }
    }
}

/// The state a sound is in after being opened with the non blocking flag,
/// or the current state of the streaming buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct OpenStateInfo {
    /// Open state of a sound.
    pub state: OpenState,
    /// Filled percentage of a stream's file buffer.
    pub percent_buffered: u32,
    /// Starving state. `true` if a stream has decoded
    /// more than the stream file buffer has ready.
    pub starving: bool,
    /// Disk is currently being accessed for this sound.
    pub disk_busy: bool,
}
