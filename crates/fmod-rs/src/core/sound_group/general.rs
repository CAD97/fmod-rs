use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    std::ptr,
};

/// # General.
impl SoundGroup {
    /// Retrieves the name of the sound group.
    pub fn get_name(&self, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_SoundGroup_GetName(
                    self.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                ))
            })
        }
    }

    raw! {
        /// Releases a soundgroup object and returns all sounds back to the
        /// master sound group.
        ///
        /// You cannot release the master [`SoundGroup`].
        pub unsafe fn raw_release(this: *mut FMOD_SOUNDGROUP) -> FMOD_RESULT {
            FMOD_SoundGroup_Release(this)
        }
    }

    // TODO: set_user_data, get_user_data

    /// Retrieves the parent System object.
    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_SoundGroup_GetSystemObject(self.as_raw(), &mut system))?;
        Ok(unsafe { System::from_raw(system) })
    }
}
