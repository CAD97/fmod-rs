use {
    crate::utils::catch_user_unwind,
    fmod::{raw::*, *},
    std::{ffi::c_void, ptr, slice},
};

/// # General.
impl Sound {
    raw! {
        /// Frees a sound object.
        ///
        /// This will stop any instances of this sound, and free the sound object
        /// and its children if it is a multi-sound object.
        ///
        /// If the sound was opened with [`Mode::NonBlocking`] and hasn't finished
        /// opening yet, it will block. Using [`Sound::get_open_state`] and checking
        /// the open state for [`OpenState::Ready`] and [`OpenState::Error`] is a
        /// good way to avoid stalls.
        pub unsafe fn raw_release(this: *mut FMOD_SOUND) -> FMOD_RESULT {
            FMOD_Sound_Release(this)
        }
    }

    // TODO: set_user_data, get_user_data

    /// Retrieves the parent System object.
    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_Sound_GetSystemObject(self.as_raw(), &mut system))?;
        Ok(unsafe { System::from_raw(system) })
    }
}

/// Callback used for user created sounds or to intercept FMOD's decoder.
pub trait PcmCallback {
    /// Invoked by:
    ///
    /// - [`System::create_sound`]
    /// - [`System::create_stream`]
    /// - [`Sound::read_data`]
    ///
    /// Use cases:
    ///
    /// - A [`Mode::OpenUser`] sound. The callback is used to write PCM data to
    ///   a user created sound when requested.
    /// - During a normal [`System::create_sound`] or [`System::create_stream`],
    ///   the read callback will be issued with decoded pcm data, allowing it to
    ///   be manipulated or copied somewhere else. The return value is ignored.
    ///
    /// The format of the sound can be retrieved with [`Sound::get_format`] from
    /// this callback. This will allow the user to determine how to interpret
    /// the data if they are not sure what format the sound is.
    fn read(sound: &Sound, data: &mut [u8]) -> Result;

    /// Invoked by:
    ///
    /// - [`System::create_sound`]
    /// - [`System::create_stream`]
    /// - [`Channel::set_position`]
    /// - [`Sound::seek_data`]
    ///
    /// Use cases:
    ///
    /// - A [`Mode::OpenUser`] sound. The callback is used to allow seeking
    ///   with the user's resource.
    /// - During a normal [`System::create_sound`] or [`System::create_stream`],
    ///   the seek callback is purely informational and the return value is
    ///   ignored.
    fn seek(sound: &Sound, subsound: i32, position: Time) -> Result;
}

pub(crate) unsafe extern "system" fn pcm_read_callback<F: PcmCallback>(
    sound: *mut FMOD_SOUND,
    data: *mut c_void,
    datalen: u32,
) -> FMOD_RESULT {
    let sound = Sound::from_raw(sound);
    let data = slice::from_raw_parts_mut(data as *mut u8, datalen as usize);
    catch_user_unwind(|| F::read(sound, data)).into_raw()
}

pub(crate) unsafe extern "system" fn pcm_setpos_callback<F: PcmCallback>(
    sound: *mut FMOD_SOUND,
    subsound: i32,
    position: u32,
    postype: FMOD_TIMEUNIT,
) -> FMOD_RESULT {
    let sound = Sound::from_raw(sound);
    let position = Time::new(position, TimeUnit::from_raw(postype));
    catch_user_unwind(|| F::seek(sound, subsound, position)).into_raw()
}

/// Callback used when a sound has finished loading, or a non blocking seek is
/// occurring.
///
/// Note that for non blocking streams a seek could occur when restarting the
/// sound after the first playthrough. This will result in a callback being
/// triggered again.
///
/// # Safety
///
/// Since this callback can occur from the async thread, there are restrictions
/// about what functions can be called during the callback. All Sound functions
/// are safe to call, except for [`Sound::set_sound_group`] and
/// [`Sound::raw_release`] (dropping a sound handle). It is also safe to call
/// [`System::get_user_data`]. The rest of the Core API and the Studio API is
/// not allowed. Calling a non-allowed function will return
/// [`Error::InvalidThread`].
pub unsafe trait NonBlockCallback {
    /// Invoked by:
    ///
    /// - [`System::create_sound`] with [`Mode::NonBlocking`] flag.
    /// - [`System::create_stream`] with [`Mode::NonBlocking`] flag.
    /// - [`Channel::set_position`] if the channel was opened with the
    ///   [`Mode::NonBlocking`] flag.
    ///
    /// Return code currently ignored.
    fn notify(sound: &Sound, result: Result) -> Result;
}

pub(crate) unsafe extern "system" fn non_block_callback<F: NonBlockCallback>(
    sound: *mut FMOD_SOUND,
    result: FMOD_RESULT,
) -> FMOD_RESULT {
    let sound = Sound::from_raw(sound);
    catch_user_unwind(|| F::notify(sound, Error::from_raw(result))).into_raw()
}
