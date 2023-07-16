use fmod::{raw::*, *};

/// # File system setup.
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
        ffi!(FMOD_System_SetFileSystem(
            self.as_raw(),
            None,
            None,
            None,
            None,
            None,
            None,
            block_align,
        ))?;
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
        ffi!(FMOD_System_SetFileSystem(
            self.as_raw(),
            Some(file::useropen::<FS>),
            Some(file::userclose::<FS>),
            Some(file::userread::<FS>),
            Some(file::userseek::<FS>),
            None,
            None,
            block_align,
        ))?;
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
        ffi!(FMOD_System_SetFileSystem(
            self.as_raw(),
            Some(file::useropen::<FS>),
            Some(file::userclose::<FS>),
            None,
            None,
            Some(file::userasyncread::<FS>),
            Some(file::userasynccancel::<FS>),
            block_align,
        ))?;
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
        ffi!(FMOD_System_AttachFileSystem(
            self.as_raw(),
            Some(file::useropen_listen::<FS>),
            Some(file::userclose_listen::<FS>),
            Some(file::userread_listen::<FS>),
            Some(file::userseek_listen::<FS>),
        ))?;
        Ok(())
    }

    /// Detach a previously [attached](Self::attach_file_system) file system
    /// listener.
    pub fn detach_file_system(&self) -> Result {
        ffi!(FMOD_System_AttachFileSystem(
            self.as_raw(),
            None,
            None,
            None,
            None,
        ))?;
        Ok(())
    }
}
