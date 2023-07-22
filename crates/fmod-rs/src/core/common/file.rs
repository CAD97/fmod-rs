//! Functionality relating to FMOD's use of the file system.

use {
    crate::utils::catch_user_unwind,
    fmod::{raw::*, *},
    std::{
        ffi::CStr,
        ffi::{c_char, c_void},
        io::{self, Read, Write},
        marker::PhantomData,
        mem::MaybeUninit,
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
    ffi!(FMOD_File_GetDiskBusy(&mut busy))?;
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

    ffi!(FMOD_File_SetDiskBusy(busy as FMOD_BOOL))?;
    Ok(())
}
/// Lock the disk busy state (see [`file::set_disk_busy`]) and unlock it
/// when dropping the returned guard object.
pub fn lock_disk_busy() -> Result<DiskBusyLock> {
    set_disk_busy(true)?;
    Ok(DiskBusyLock { _priv: () })
}

#[derive(Debug)]
/// Drop guard for file busy state.
///
/// While you have this, FMOD won't do any file IO.
pub struct DiskBusyLock {
    _priv: (),
}

impl Drop for DiskBusyLock {
    fn drop(&mut self) {
        match set_disk_busy(false) {
            Ok(()) => (),
            Err(error) => {
                whoops!("Error unlocking file busy state: {error}");
            },
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A writable handle into an uninitialized memory buffer.
///
/// [`std::io::BorrowedCursor`] but specialized to FMOD's FFI.
pub struct FileBuffer<'a> {
    buffer: &'a mut [MaybeUninit<u8>],
    written: &'a mut u32,
}

impl<'a> FileBuffer<'a> {
    /// The total capacity of the buffer.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// The number of bytes that have been written to the buffer.
    pub fn written(&self) -> usize {
        ix!(*self.written)
    }

    /// Whether the buffer is completely filled.
    pub fn is_full(&self) -> bool {
        self.written() == self.capacity()
    }

    /// The unfilled part of the buffer.
    pub fn unfilled(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.buffer[ix!(*self.written)..]
    }

    /// Advance the cursor by asserting that `n` additional bytes have been filled.
    ///
    /// # Safety
    ///
    /// At least the first `n` in the unfilled portion of the buffer must have
    /// been properly initialized and filled.
    pub unsafe fn advance(&mut self, n: usize) {
        *self.written += n as u32;
    }

    /// Fill the buffer from a reader.
    ///
    /// Both filling up the buffer (`ErrorKind::WriteZero`) and exhausting the
    /// reader (`ErrorKind::UnexpectedEof`) are considered successes and return
    /// `Ok(())`. Check [`written`](Self::written) to see how many bytes have
    /// been written and [`is_full`](Self::is_full) to see if the buffer has
    /// been completely filled.
    ///
    /// This behavior was chosen to be the most useful for implementing
    /// [`SyncFileSystem`] and [`AsyncFileSystem`], as reads through those
    /// traits should succeed if the read itself was successful, and a success
    /// gets automatically translated to [`Error::FileEof`] when appropriate.
    pub fn fill_from(&mut self, reader: &mut (impl Read + ?Sized)) -> io::Result<()> {
        let result;

        #[cfg(feature = "unstable")]
        {
            let mut buf = io::BorrowedBuf::from(self.unfilled());
            result = reader.read_buf_exact(buf.unfilled());
            let written = buf.filled().len();
            unsafe { self.advance(written) };
        };

        #[cfg(not(feature = "unstable"))]
        {
            result = io::copy(reader, self);
        }

        match result {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::WriteZero => Ok(()),
            Err(e) => yeet!(e),
        }
    }
}

impl<'a> Write for FileBuffer<'a> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        unsafe {
            let data = &*(data as *const [u8] as *const [MaybeUninit<u8>]);
            let buf = self.unfilled();
            let len = usize::min(buf.len(), data.len());
            buf[..len].copy_from_slice(&data[..len]);
            self.advance(len);
            Ok(len)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A handle to an FMOD asynchronous file read operation, received from
/// [`AsyncFileSystem`].
///
/// When servicing the async read operation, read from
/// [`handle`](Self::handle) at the given [`offset`](Self::offset) for
/// [`size`](Self::size) bytes into [`buffer`](Self::buffer). Then call
/// [`done`](Self::done) with the [`Result`] of the operation.
///
/// # Safety
///
/// This structure must not be used after calling [`done`](Self::done) or
/// the read operation has been [`cancel`led](file::AsyncFileSystem::cancel).
#[derive(Debug)]
pub struct AsyncReadInfo<File> {
    raw: *mut FMOD_ASYNCREADINFO,
    _phantom: PhantomData<*mut *mut File>,
}

unsafe impl<File> Send for AsyncReadInfo<File> where for<'a> &'a mut File: Send {}
unsafe impl<File> Sync for AsyncReadInfo<File> where for<'a> &'a mut File: Sync {}

impl<File> Copy for AsyncReadInfo<File> {}
impl<File> Clone for AsyncReadInfo<File> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<File> Eq for AsyncReadInfo<File> {}
impl<File> PartialEq for AsyncReadInfo<File> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

#[allow(clippy::missing_safety_doc)]
impl<File> AsyncReadInfo<File> {
    raw! {
        pub const fn from_raw(raw: *mut FMOD_ASYNCREADINFO) -> Self {
            Self { raw, _phantom: PhantomData }
        }
    }

    raw! {
        pub const fn into_raw(self) -> *mut FMOD_ASYNCREADINFO {
            self.raw
        }
    }

    /// Gets the unique address identifier for this async request.
    ///
    /// Note that addresses may be reused after the request is completed.
    pub fn addr(self) -> usize {
        self.raw as usize
    }

    /// File handle that was provided by [`FileSystem::open`].
    ///
    /// # Safety
    ///
    /// Returns an unbound lifetime; borrow access patterns are unchecked.
    pub unsafe fn handle<'a>(self) -> Pin<&'a File> {
        Pin::new_unchecked(&*self.handle_ptr())
    }

    /// File handle that was provided by [`FileSystem::open`].
    ///
    /// # Safety
    ///
    /// Returns an unbound lifetime; borrow access patterns are unchecked.
    pub unsafe fn handle_mut<'a>(self) -> Pin<&'a mut File> {
        Pin::new_unchecked(&mut *self.handle_ptr())
    }

    /// File handle that was provided by [`FileSystem::open`].
    pub unsafe fn handle_ptr(self) -> *mut File {
        (*self.raw).handle.cast()
    }

    /// Byte offset within the file where the read operation should occur.
    pub unsafe fn offset(self) -> u32 {
        (*self.raw).offset
    }

    /// Number of bytes to read.
    pub unsafe fn size(self) -> u32 {
        (*self.raw).sizebytes
    }

    /// Priority hint for how quickly this operation should be serviced
    /// where 0 represents low importance and 100 represents extreme
    /// importance. This could be used to prioritize the read order of a
    /// file job queue for example. FMOD decides the importance of the read
    /// based on if it could degrade audio or not.
    pub unsafe fn priority(self) -> i32 {
        (*self.raw).priority
    }

    /// Buffer to read data into.
    ///
    /// # Safety
    ///
    /// Returns an unbound lifetime; borrow access patterns are unchecked.
    pub unsafe fn buffer_mut<'a>(self) -> FileBuffer<'a> {
        let ptr = (*self.raw).buffer;
        let len = self.size();
        let buffer = slice::from_raw_parts_mut(ptr.cast(), len as usize);
        let written = &mut (*self.raw).bytesread;
        FileBuffer { buffer, written }
    }

    /// Completion function to signal the async read is done.
    ///
    /// Relevant result codes to use with this function include:
    ///
    /// - `Ok`: Read was successful.
    /// - [`Error::FileDiskEjected`]: Read was cancelled before being serviced.
    /// - [`Error::FileBad`]: Read operation failed for any other reason.
    ///
    /// A result of `Ok` will be automatically translated to [`Error::FileEof`]
    /// if the number of bytes read is less than the requested number of bytes.
    ///
    /// # Safety
    ///
    /// When calling this the implementation must ensure that all copies of
    /// this [`AsyncReadInfo`] are relinquished.
    pub unsafe fn done(self, mut result: Result) {
        if result.is_ok() && !self.buffer_mut().unfilled().is_empty() {
            result = Err(Error::FileEof);
        }
        (*self.raw).done.unwrap_unchecked()(self.raw, result.into_raw());
    }
}

/// Data returned from [`FileSystem::open`].
#[derive(Debug)]
pub struct FileOpenInfo<File> {
    /// The file handle.
    pub handle: Pin<Box<File>>,
    /// The size of the file in bytes.
    pub file_size: usize,
}

/// Callbacks to implement all file I/O instead of using the platform native
/// method.
#[allow(clippy::missing_safety_doc)]
pub trait FileSystem {
    /// The file handle used by this file system.
    type File: Send + Sync;

    /// Callback for opening a file.
    ///
    /// Return the appropriate error such as [Error::FileNotFound] if the
    /// file fails to open.
    fn open(name: &CStr) -> Result<FileOpenInfo<Self::File>>;

    /// Callback for closing a file.
    ///
    /// Close any user created file handle and perform any cleanup necessary
    /// for the file here. Default implemented to just drop the file handle.
    fn close(file: Pin<Box<Self::File>>) -> Result {
        drop(file);
        Ok(())
    }
}

pub(crate) unsafe extern "system" fn useropen<FS: FileSystem>(
    name: *const c_char,
    filesize: *mut u32,
    handle: *mut *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| {
        let name = CStr::from_ptr(name);
        let file = FS::open(name)?;
        *filesize = file.file_size.try_into().map_err(|_| Error::FileBad)?;
        *handle = Box::into_raw(Pin::into_inner_unchecked(file.handle)).cast();
        Ok(())
    })
    .into_raw()
}

pub(crate) unsafe extern "system" fn userclose<FS: FileSystem>(
    handle: *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let file = Pin::new_unchecked(Box::from_raw(handle.cast()));
    catch_user_unwind(|| FS::close(file)).into_raw()
}

/// Callbacks to implement all file I/O instead of using the platform native
/// method.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait SyncFileSystem: FileSystem {
    /// Callback for reading from a file.
    ///
    /// The entire buffer should be filled with contents from the file. If
    /// there is not enough data to provide the requested number of bytes,
    /// a return value of `Ok` will be automatically translated to
    /// <code>Err([Error::FileEof])</code>.
    fn read(file: Pin<&mut Self::File>, buffer: FileBuffer<'_>) -> Result;

    /// Callback for seeking within a file.
    fn seek(file: Pin<&mut Self::File>, pos: u32) -> Result;
}

pub(crate) unsafe extern "system" fn userread<FS: SyncFileSystem>(
    handle: *mut c_void,
    buffer: *mut c_void,
    sizebytes: u32,
    bytesread: *mut u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| {
        let buffer = slice::from_raw_parts_mut(buffer.cast(), ix!(sizebytes));
        let file = Pin::new_unchecked(&mut *handle.cast());

        *bytesread = 0; // ensure this starts at 0 in case FMOD doesn't
        let buf = FileBuffer {
            buffer,
            written: &mut *bytesread,
        };

        FS::read(file, buf)?;
        if *bytesread < sizebytes {
            Err(Error::FileEof)
        } else {
            Ok(())
        }
    })
    .into_raw()
}

pub(crate) unsafe extern "system" fn userseek<FS: SyncFileSystem>(
    handle: *mut c_void,
    pos: u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let file = Pin::new_unchecked(&mut *handle.cast());
    catch_user_unwind(|| FS::seek(file, pos)).into_raw()
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
    /// # Safety
    ///
    /// Before returning from this callback the implementation must ensure
    /// that all copies of `info` are relinquished.
    unsafe fn cancel(info: AsyncReadInfo<Self::File>) -> Result;
}

pub(crate) unsafe extern "system" fn userasyncread<FS: AsyncFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    (*info).bytesread = 0; // ensure this starts at 0 in case FMOD doesn't
    catch_user_unwind(|| FS::read(AsyncReadInfo::from_raw(info))).into_raw()
}

pub(crate) unsafe extern "system" fn userasynccancel<FS: AsyncFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| FS::cancel(AsyncReadInfo::from_raw(info))).into_raw()
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

pub(crate) unsafe extern "system" fn useropen_listen<FS: ListenFileSystem>(
    name: *const c_char,
    filesize: *mut u32,
    handle: *mut *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let name = CStr::from_ptr(name);
    catch_user_unwind(|| Ok(FS::open(name, *filesize, (*handle) as usize))).into_raw()
}

pub(crate) unsafe extern "system" fn userclose_listen<FS: ListenFileSystem>(
    handle: *mut c_void,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| Ok(FS::close(handle as usize))).into_raw()
}

pub(crate) unsafe extern "system" fn userread_listen<FS: ListenFileSystem>(
    handle: *mut c_void,
    buffer: *mut c_void,
    sizebytes: u32,
    bytesread: *mut u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    let buffer = slice::from_raw_parts_mut(buffer.cast(), ix!(*bytesread));
    let eof = buffer.len() < ix!(sizebytes);
    catch_user_unwind(|| Ok(FS::read(handle as usize, buffer, eof))).into_raw()
}

pub(crate) unsafe extern "system" fn userseek_listen<FS: ListenFileSystem>(
    handle: *mut c_void,
    pos: u32,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| Ok(FS::seek(handle as usize, pos))).into_raw()
}

/// 'Piggyback' on FMOD file reading routines to capture data as it's read.
pub trait AsyncListenFileSystem: ListenFileSystem {
    #[allow(clippy::missing_safety_doc)]
    /// Callback for after an async read operation.
    ///
    /// # Safety
    ///
    /// The `AsyncReadInfo` must be valid for at least the duration of this callback.
    unsafe fn async_read(info: AsyncReadInfo<()>) {
        let _ = info;
    }

    /// Callback for after an async cancel operation.
    ///
    /// # Safety
    ///
    /// The `AsyncReadInfo` must be valid for at least the duration of this callback.
    unsafe fn async_cancel(info: AsyncReadInfo<()>) {
        let _ = info;
    }
}

pub(crate) unsafe extern "system" fn userasyncread_listen<FS: AsyncListenFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| Ok(FS::async_read(AsyncReadInfo::from_raw(info)))).into_raw()
}

pub(crate) unsafe extern "system" fn userasynccancel_listen<FS: AsyncListenFileSystem>(
    info: *mut FMOD_ASYNCREADINFO,
    _userdata: *mut c_void,
) -> FMOD_RESULT {
    catch_user_unwind(|| Ok(FS::async_cancel(AsyncReadInfo::from_raw(info)))).into_raw()
}
