use {
    crate::utils::catch_user_unwind,
    fmod::{raw::*, *},
    std::{
        ffi::CStr,
        mem::MaybeUninit,
        os::raw::{c_char, c_void},
        panic::AssertUnwindSafe,
        pin::Pin,
        slice,
    },
};

/// Information function to retrieve the state of FMOD disk access.
///
/// Do not use this function to synchronize your own reads with, as due to
/// timing, you might call this function and it says false = it is not busy,
/// but the split second after calling this function, internally FMOD might
/// set it to busy. Use [file::set_disk_busy] for proper mutual exclusion as
/// it uses semaphores.
pub fn get_disk_busy() -> Result<bool> {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();

    let mut busy = 0;
    fmod_try!(FMOD_File_GetDiskBusy(&mut busy));
    Ok(busy != 0)
}

/// Sets the busy state for disk access ensuring mutual exclusion of file
/// operations.
///
/// If file IO is currently being performed by FMOD this function will block
/// until it has completed.
///
/// This function should be called in pairs once to set the state, then
/// again to clear it once complete.
pub fn set_disk_busy(busy: bool) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();

    fmod_try!(FMOD_File_SetDiskBusy(if busy { 1 } else { 0 }));
    Ok(())
}

#[derive(Debug)]
/// Drop guard for file busy state.
///
/// While you have this, FMOD won't do any file IO.
pub struct FileBusyGuard {
    _priv: (),
}

impl Drop for FileBusyGuard {
    fn drop(&mut self) {
        match set_disk_busy(false) {
            Ok(()) => (),
            Err(error) => {
                whoops!("Error unlocking file busy state: {error}");
            },
        }
    }
}

/// Lock the disk busy state (see [`file::set_disk_busy`]) and unlock it
/// when dropping the returned guard object.
pub fn lock_disk_busy() -> Result<FileBusyGuard> {
    set_disk_busy(true)?;
    Ok(FileBusyGuard { _priv: () })
}

/// Callbacks to implement all file I/O instead of using the platform native
/// method.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait FileSystem {
    type File;

    /// Callback for opening a file.
    ///
    /// Return the appropriate error such as [Error::FileNotFound] if the
    /// file fails to open.
    fn open(name: &CStr) -> Result<(u32, Pin<Box<Self::File>>)>;

    /// Callback for closing a file.
    ///
    /// Close any user created file handle and perform any cleanup necessary
    /// for the file here.
    fn close(file: Pin<Box<Self::File>>) -> Result;
}

/// Callbacks to implement all file I/O instead of using the platform native
/// method.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait SyncFileSystem: FileSystem {
    /// Callback for reading from a file.
    ///
    /// The entire buffer should be filled with contents from the file. If
    /// there is not enough data to read the requested number of bytes,
    /// return the number of bytes that were read; this is interpreted as an
    /// EOF condition.
    fn read(file: Pin<&mut Self::File>, buffer: &mut [MaybeUninit<u8>]) -> Result<u32>;

    /// Callback for seeking within a file.
    fn seek(file: Pin<&mut Self::File>, pos: u32) -> Result;
}

/// Callbacks to implement all file I/O instead of using the platform native
/// method.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait AsyncFileSystem: FileSystem {
    /// Callback for reading from a file asynchronously.
    ///
    /// This callback allows you to accept a file I/O request without
    /// servicing it immediately. The callback can queue or store the
    /// [`AsyncReadInfo`] handle, so that a 'servicing routine' can read
    /// the data and mark the job as done.
    ///
    /// Marking an asynchronous job as 'done' outside of this callback can
    /// be done by calling the [`AsyncReadInfo::done`] function with the
    /// file read result.
    ///
    /// If the servicing routine is processed in the same thread as the
    /// thread that invokes this callback (for example the thread that calls
    /// [`System::create_sound`] or [`System::create_stream`]), a deadlock
    /// will occur because while [`System::create_sound`] or
    /// [`System::create_stream`] waits for the file data, the servicing
    /// routine in the main thread won't be able to execute.
    ///
    /// This typically means an outside servicing routine should typically
    /// be run in a separate thread.
    ///
    /// The read request can be queued or stored and this callback can
    /// return immediately with `Ok`. Returning an error at this point will
    /// cause FMOD to stop what it was doing and return back to the caller.
    /// If it is from FMOD's stream thread, the stream will typically stop.
    unsafe fn read(info: AsyncReadInfo<Self::File>) -> Result;

    /// Callback for cancelling a pending asynchronous read.
    ///
    /// This callback is called to stop/release or shut down the resource
    /// that is holding the file, for example: releasing a [Sound] stream.
    ///
    /// Before returning from this callback the implementation must ensure
    /// that all copies of [`info`] are relinquished.
    unsafe fn cancel(info: AsyncReadInfo<Self::File>) -> Result;
}

pub(crate) unsafe extern "system" fn useropen<FS: FileSystem>(
    name: *const c_char,
    filesize: *mut u32,
    handle: *mut *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let name = CStr::from_ptr(name);
    match catch_user_unwind(|| FS::open(name)) {
        Some(Ok((size, file))) => {
            *filesize = size;
            *handle = Box::into_raw(Pin::into_inner_unchecked(file)).cast();
            FMOD_OK
        },
        Some(Err(err)) => err.into_raw(),
        None => FMOD_ERR_FILE_BAD,
    }
}

pub(crate) unsafe extern "system" fn userclose<FS: FileSystem>(
    handle: *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let file = Pin::new_unchecked(Box::from_raw(handle.cast()));
    let file = AssertUnwindSafe(file);
    match catch_user_unwind(|| FS::close({ file }.0)) {
        Some(Ok(())) => FMOD_OK,
        Some(Err(err)) => err.into_raw(),
        None => FMOD_ERR_FILE_BAD,
    }
}

pub(crate) unsafe extern "system" fn userread<FS: SyncFileSystem>(
    handle: *mut c_void,
    buffer: *mut c_void,
    sizebytes: u32,
    bytesread: *mut u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let buffer = slice::from_raw_parts_mut(buffer.cast(), sizebytes as usize);
    let buffer = AssertUnwindSafe(buffer);

    let file = Pin::new_unchecked(&mut *handle.cast());
    let file = AssertUnwindSafe(file);

    match catch_user_unwind(|| FS::read({ file }.0, { buffer }.0)) {
        Some(Ok(read)) => {
            *bytesread = read;
            if read < sizebytes {
                FMOD_ERR_FILE_EOF
            } else {
                FMOD_OK
            }
        },
        Some(Err(err)) => err.into_raw(),
        None => FMOD_ERR_FILE_BAD,
    }
}

pub(crate) unsafe extern "system" fn userseek<FS: SyncFileSystem>(
    handle: *mut c_void,
    pos: u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let file = Pin::new_unchecked(&mut *handle.cast());
    let file = AssertUnwindSafe(file);

    match catch_user_unwind(|| FS::seek({ file }.0, pos)) {
        Some(Ok(())) => FMOD_OK,
        Some(Err(err)) => err.into_raw(),
        None => FMOD_ERR_FILE_BAD,
    }
}

pub(crate) unsafe extern "system" fn userasyncread<FS: AsyncFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    match catch_user_unwind(|| FS::read(AsyncReadInfo::from_raw(info))) {
        Some(Ok(())) => FMOD_OK,
        Some(Err(err)) => err.into_raw(),
        None => FMOD_ERR_FILE_BAD,
    }
}

pub(crate) unsafe extern "system" fn userasynccancel<FS: AsyncFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    match catch_user_unwind(|| FS::cancel(AsyncReadInfo::from_raw(info))) {
        Some(Ok(())) => FMOD_OK,
        Some(Err(err)) => err.into_raw(),
        None => FMOD_ERR_FILE_BAD,
    }
}

/// 'Piggyback' on FMOD file reading routines to capture data as it's read.
pub trait ListenFileSystem {
    /// Callback for after a file is opened.
    fn open(name: &CStr, size: u32, handle: usize) {
        let _ = (name, size, handle);
    }
    /// Callback for after a file is closed.
    fn close(handle: usize) {
        let _ = handle;
    }
    /// Callback for after a read operation.
    fn read(handle: usize, buffer: &[u8], eof: bool) {
        let _ = (handle, buffer, eof);
    }
    /// Callback for after a seek operation.
    fn seek(handle: usize, pos: u32) {
        let _ = (handle, pos);
    }
}

#[allow(clippy::missing_safety_doc)]
pub trait AsyncListenFileSystem: ListenFileSystem {
    unsafe fn async_read(info: AsyncReadInfo<()>) {
        let _ = info;
    }
    unsafe fn async_cancel(info: AsyncReadInfo<()>) {
        let _ = info;
    }
}

pub(crate) unsafe extern "system" fn useropen_listen<FS: ListenFileSystem>(
    name: *const c_char,
    filesize: *mut u32,
    handle: *mut *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let name = CStr::from_ptr(name);
    FS::open(name, *filesize, (*handle) as usize);
    FMOD_OK
}

pub(crate) unsafe extern "system" fn userclose_listen<FS: ListenFileSystem>(
    handle: *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    FS::close(handle as usize);
    FMOD_OK
}

pub(crate) unsafe extern "system" fn userread_listen<FS: ListenFileSystem>(
    handle: *mut c_void,
    buffer: *mut c_void,
    sizebytes: u32,
    bytesread: *mut u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let buffer = slice::from_raw_parts_mut(buffer.cast(), *bytesread as usize);
    let eof = buffer.len() < sizebytes as usize;
    catch_user_unwind(|| FS::read(handle as usize, buffer, eof));
    FMOD_OK
}

pub(crate) unsafe extern "system" fn userseek_listen<FS: ListenFileSystem>(
    handle: *mut c_void,
    pos: u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| FS::seek(handle as usize, pos));
    FMOD_OK
}

pub(crate) unsafe extern "system" fn userasyncread_listen<FS: AsyncListenFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| FS::async_read(AsyncReadInfo::from_raw(info)));
    FMOD_OK
}

pub(crate) unsafe extern "system" fn userasynccancel_listen<FS: AsyncListenFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| FS::async_cancel(AsyncReadInfo::from_raw(info)));
    FMOD_OK
}
