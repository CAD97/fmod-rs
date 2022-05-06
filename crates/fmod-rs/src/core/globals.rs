pub mod memory {
    use {
        crate::utils::{catch_user_unwind, str_from_nonnull_unchecked},
        fmod::{raw::*, *},
        std::{
            alloc::{alloc, dealloc, realloc, Layout},
            mem::{self, MaybeUninit},
            os::raw::{c_char, c_uint, c_void},
            ptr,
        },
    };

    /// Information on the memory usage of FMOD.
    pub struct MemoryStats {
        /// Currently allocated memory at time of call.
        pub current_alloced: i32,
        /// Maximum allocated memory since [System::init] or
        /// [memory::initialize].
        pub max_alloced: i32,
    }

    /// Returns information on the memory usage of FMOD.
    ///
    /// `blocking` indicates whether to favour speed or accuracy. Specifying
    /// `true` for this parameter will flush the DSP network to make sure all
    /// queued allocations happen immediately, which can be costly.
    ///
    /// This information is byte accurate and counts all allocs and frees
    /// internally. This is useful for determining a fixed memory size to make
    /// FMOD work within for fixed memory machines such as consoles.
    ///
    /// Note that if using [memory::initialize], the memory usage will be
    /// slightly higher than without it, as FMOD has to have a small amount of
    /// memory overhead to manage the available memory.
    pub fn get_stats(blocking: bool) -> Result<MemoryStats> {
        // prevent racing System init
        let _lock = GLOBAL_SYSTEM_STATE.read();

        let mut current_alloced = 0;
        let mut max_alloced = 0;
        fmod_try!(FMOD_Memory_GetStats(
            &mut current_alloced,
            &mut max_alloced,
            if blocking { 1 } else { 0 }
        ));

        Ok(MemoryStats {
            current_alloced,
            max_alloced,
        })
    }

    /// Specifies that FMOD should use its default method to allocate and free
    /// memory.
    ///
    /// To specify a custom memory management strategy, use a different
    /// `initialize` in [this module][memory].
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    pub unsafe fn initialize() -> Result {
        fmod_try!(FMOD_Memory_Initialize(
            ptr::null_mut(),
            0,
            None,
            None,
            None,
            0
        ));
        Ok(())
    }

    /// Specifies for FMOD to allocate and free memory in a user supplied
    /// buffer.
    ///
    /// The buffer is truncated to a multiple of 512 bytes.
    ///
    /// To find out the required fixed size find out the maximum RAM usage at
    /// any one time with [memory::get_stats].
    ///
    /// If you specify a fixed size pool that is too small, FMOD will return
    /// [Error::Memory] when the limit of the fixed size pool is exceeded. At
    /// this point, it's possible that FMOD may become unstable. To maintain
    /// stability, do not allow FMOD to run out of memory.
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    pub unsafe fn initialize_pool(pool: &'static mut [MaybeUninit<u8>]) -> Result {
        let pool_len = pool.len() % 512;
        fmod_try!(FMOD_Memory_Initialize(
            pool.as_mut_ptr().cast(),
            pool_len.try_into().unwrap_or(i32::MAX),
            None,
            None,
            None,
            0
        ));
        Ok(())
    }

    /// User callbacks for FMOD to allocate and free memory.
    ///
    /// Callback implementations must be thread safe.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait FmodAlloc {
        /// Memory allocation callback compatible with ANSI malloc.
        ///
        /// Returning an aligned pointer, of 16 byte alignment is recommended
        /// for performance reasons.
        ///
        /// <dl>
        /// <dt>size</dt><dd>Size in bytes of the memory block to be allocated
        /// and returned.</dd>
        /// <dt>kind</dt><dd>Type of memory allocation.</dd>
        /// <dt>source</dt><dd>String with the FMOD source code filename and
        /// line number in it. Only provided in logging versions of FMOD.</dd>
        /// </dl>
        fn alloc(size: u32, kind: MemoryType, source: Option<&str>) -> *mut u8;

        /// Memory free callback compatible with ANSI free.
        ///
        /// <dl>
        /// <dt>ptr</dt><dd>Pre-existing block of memory to be freed.</dd>
        /// <dt>kind</dt><dd>Type of memory to be freed.</dd>
        /// <dt>source</dt><dd>String with the FMOD source code filename and
        /// line number in it. Only provided in logging versions of FMOD.</dd>
        /// </dl>
        unsafe fn free(ptr: *mut u8, kind: MemoryType, source: Option<&str>);
    }

    /// User callbacks for FMOD to allocate and free memory.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait FmodRealloc: FmodAlloc {
        /// Memory reallocation callback compatible with ANSI realloc.
        ///
        /// When allocating new memory, the contents of the old memory block
        /// must be preserved.
        ///
        /// Returning an aligned pointer, of 16 byte alignment is recommended
        /// for performance reasons.
        ///
        /// <dl>
        /// <dt>ptr</dt><dd>Block of memory to be resized. If this is null, then
        /// a new block of memory is allocated and no memory is freed.</dd>
        /// <dt>size</dt><dd>Size of the memory to be reallocated.</dd>
        /// <dt>source</dt><dd>String with the FMOD source code filename and
        /// line number in it. Only provided in logging versions of FMOD.</dd>
        /// </dl>
        unsafe fn realloc(
            ptr: *mut u8,
            size: u32,
            kind: MemoryType,
            source: Option<&str>,
        ) -> *mut u8;
    }

    unsafe extern "system" fn useralloc<A: FmodAlloc>(
        size: c_uint,
        kind: FMOD_MEMORY_TYPE,
        source: *const c_char,
    ) -> *mut c_void {
        catch_user_unwind(|| {
            // SAFETY: these strings are produced directly by FMOD, so they
            // should *actually* be guaranteed to be UTF-8 like FMOD claims.
            let source = ptr::NonNull::new(source as *mut _).map(|x| str_from_nonnull_unchecked(x));
            A::alloc(size, MemoryType::from_raw(kind), source).cast()
        })
        .unwrap_or(ptr::null_mut())
    }

    unsafe extern "system" fn userrealloc<A: FmodRealloc>(
        ptr: *mut c_void,
        size: c_uint,
        kind: FMOD_MEMORY_TYPE,
        source: *const c_char,
    ) -> *mut c_void {
        catch_user_unwind(|| {
            // SAFETY: these strings are produced directly by FMOD, so they
            // should *actually* be guaranteed to be UTF-8 like FMOD claims.
            let source = ptr::NonNull::new(source as *mut _).map(|x| str_from_nonnull_unchecked(x));
            A::realloc(ptr.cast(), size, MemoryType::from_raw(kind), source).cast()
        })
        .unwrap_or(ptr::null_mut())
    }

    unsafe extern "system" fn userfree<A: FmodAlloc>(
        ptr: *mut c_void,
        kind: FMOD_MEMORY_TYPE,
        source: *const c_char,
    ) {
        catch_user_unwind(|| {
            // SAFETY: these strings are produced directly by FMOD, so they
            // should *actually* be guaranteed to be UTF-8 like FMOD claims.
            let source = ptr::NonNull::new(source as *mut _).map(|x| str_from_nonnull_unchecked(x));
            A::free(ptr.cast(), MemoryType::from_raw(kind), source)
        })
        .unwrap_or_default()
    }

    /// Specifies for FMOD to allocate and free memory through user supplied
    /// callbacks.
    ///
    /// Callbacks will be used only for memory types specified by the
    /// `mem_type_flags` parameter.
    ///
    /// `realloc` is implemented via an allocation of the new size, copy from
    /// old address to new, then a free of the old address.
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    pub unsafe fn initialize_alloc<A: FmodAlloc>(mem_type_flags: MemoryType) -> Result {
        fmod_try!(FMOD_Memory_Initialize(
            ptr::null_mut(),
            0,
            Some(useralloc::<A>),
            None,
            Some(userfree::<A>),
            mem_type_flags.into_raw(),
        ));
        Ok(())
    }

    /// Specifies for FMOD to allocate and free memory through user supplied
    /// callbacks.
    ///
    /// Callbacks will be used only for memory types specified by the
    /// `mem_type_flags` parameter.
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    //
    // FEAT(specialization): automatically do this via specialization
    pub unsafe fn initialize_realloc<A: FmodRealloc>(mem_type_flags: MemoryType) -> Result {
        fmod_try!(FMOD_Memory_Initialize(
            ptr::null_mut(),
            0,
            Some(useralloc::<A>),
            Some(userrealloc::<A>),
            Some(userfree::<A>),
            mem_type_flags.into_raw(),
        ));
        Ok(())
    }

    /// Specifies for FMOD to allocate and free memory via the Rust global
    /// allocator.
    ///
    /// *This has overhead:* FMOD expects an ANSI `malloc`/`free` interface
    /// without a size parameter to `free`. Rust's global allocator requires
    /// `Layout` to be given on `dealloc`, so filling an ANSI `free` shaped hole
    /// with Rust's `dealloc` requires storing size information somewhere.
    ///
    /// This bridge takes the traditional approach used by allocators and
    /// over-allocates from what was requested to store the layout information
    /// in the allocation. Unfortunately, since most general purpose allocators
    /// have an ANSI-compatible interface, this is purely a waste in most
    /// situations, requried just to satisfy the Rust middleman.
    ///
    /// In most cases, you should prefer leaving the defaults (FMOD will use the
    /// system allocator) or one of the other [`memory::initialize`] variants
    /// which don't require this overhead.
    pub enum FmodAllocViaRust {}

    unsafe impl FmodAlloc for FmodAllocViaRust {
        fn alloc(size: u32, _: MemoryType, _: Option<&str>) -> *mut u8 {
            unsafe {
                let layout = Layout::from_size_align_unchecked(size as usize + 16, 16);
                let ptr = alloc(layout);
                if ptr.is_null() {
                    return ptr;
                }
                ::static_assertions::const_assert!(mem::size_of::<usize>() <= 16);
                ptr.cast::<usize>().write(layout.size());
                ptr.add(16)
            }
        }

        unsafe fn free(ptr: *mut u8, _: MemoryType, _: Option<&str>) {
            let ptr = ptr.sub(16);
            let size = ptr.cast::<usize>().read();
            let layout = Layout::from_size_align_unchecked(size, 16);
            dealloc(ptr, layout)
        }
    }

    unsafe impl FmodRealloc for FmodAllocViaRust {
        unsafe fn realloc(ptr: *mut u8, size: u32, _: MemoryType, _: Option<&str>) -> *mut u8 {
            let new_size = size as usize + 16;
            let ptr = ptr.sub(16);
            let old_size = ptr.cast::<usize>().read();
            let old_layout = Layout::from_size_align_unchecked(old_size, 16);
            let ptr = realloc(ptr, old_layout, new_size);
            if ptr.is_null() {
                return ptr;
            }
            ptr.cast::<usize>().write(new_size);
            ptr.add(16)
        }
    }
}

pub mod debug {
    use std::ffi::CStr;

    use {
        crate::utils::{catch_user_unwind, str_from_nonnull_unchecked},
        fmod::{raw::*, *},
        std::{
            os::raw::{c_char, c_int},
            ptr,
            sync::atomic::{AtomicBool, Ordering},
        },
    };

    /// Callback for debug messages when using the logging version of FMOD.
    ///
    /// This callback will fire directly from the log line, as such it can be
    /// from any thread.
    pub trait FmodDebug {
        /// Callback for debug messages when using the logging version of FMOD.
        ///
        /// <dl>
        /// <dt>flags</dt><dd>Flags which detail the level and type of this log.</dd>
        /// <dt>file</dt><dd>Source code file name where the message originated.</dd>
        /// <dt>line</dt><dd>Source code line number where the message originated.</dd>
        /// <dt>func</dt><dd>Class and function name where the message originated.</dd>
        /// <dt>message</dt><dd>Actual debug message associated with the callback.</dd>
        /// </dl>
        fn log(
            flags: DebugFlags,
            file: Option<&str>,
            line: i32,
            func: Option<&str>,
            message: Option<&str>,
        ) -> Result<()>;
    }

    unsafe extern "system" fn callback<D: FmodDebug>(
        flags: FMOD_DEBUG_FLAGS,
        file: *const c_char,
        line: c_int,
        func: *const c_char,
        msg: *const c_char,
    ) -> FMOD_RESULT {
        catch_user_unwind(|| {
            let flags = DebugFlags::from_raw(flags);
            // SAFETY: these strings are produced directly by FMOD, so they
            // should *actually* be guaranteed to be UTF-8 like FMOD claims.
            let file = ptr::NonNull::new(file as *mut _)
                .map(|x| str_from_nonnull_unchecked(x))
                .map(str::trim_end);
            let func = ptr::NonNull::new(func as *mut _)
                .map(|x| str_from_nonnull_unchecked(x))
                .map(str::trim_end);
            let message = ptr::NonNull::new(msg as *mut _)
                .map(|x| str_from_nonnull_unchecked(x))
                .map(str::trim_end);
            D::log(flags, file, line, func, message)
        })
        .unwrap_or(Err(Error::InternalRs))
        .map_or_else(Error::into_raw, |()| FMOD_OK)
    }

    static DEBUG_LAYER_INITIALIZED: AtomicBool = AtomicBool::new(false);

    /// Specify the level and delivery method of log messages when using the
    /// logging version of FMOD.
    ///
    /// This method initializes logs to go to the default log location per
    /// platform, i.e. Visual Studio output window, stderr, LogCat, etc.
    ///
    /// This function will return [Error::Unsupported] when using the
    /// non-logging (release) versions of FMOD.
    ///
    /// The logging version of FMOD can be recognized by the 'L' suffix in the
    /// library name, fmodL.dll or libfmodL.so for instance.
    ///
    /// Note that:
    /// - [DebugFlags::LevelLog] produces informational, warning and error
    ///   messages.
    /// - [DebugFlags::LevelWarning] produces warnings and error messages.
    /// - [DebugFlags::LevelError] produces error messages only.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    /// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
    /// the <code>fmod_debug_is_tracing</code> feature flag is set. Manually
    /// initializing FMOD debugging will override this behavior.
    /// </pre>
    pub fn initialize(flags: DebugFlags) -> Result {
        // prevent racing System init
        let _lock = GLOBAL_SYSTEM_STATE.read();
        // suppress default debug init
        DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

        fmod_try!(FMOD_Debug_Initialize(
            flags.into_raw(),
            DebugMode::Tty.into_raw(),
            None,
            ptr::null()
        ));
        Ok(())
    }

    /// Specify the level and delivery method of log messages when using the
    /// logging version of FMOD.
    ///
    /// This method initializes logs to go to call specified callback with log
    /// information.
    ///
    /// This function will return [Error::Unsupported] when using the
    /// non-logging (release) versions of FMOD.
    ///
    /// The logging version of FMOD can be recognized by the 'L' suffix in the
    /// library name, fmodL.dll or libfmodL.so for instance.
    ///
    /// Note that:
    /// - [DebugFlags::LevelLog] produces informational, warning and error
    ///   messages.
    /// - [DebugFlags::LevelWarning] produces warnings and error messages.
    /// - [DebugFlags::LevelError] produces error messages only.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    /// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
    /// the <code>fmod_debug_is_tracing</code> feature flag is set. Manually
    /// initializing FMOD debugging will override this behavior.
    /// </pre>
    pub fn initialize_callback<D: FmodDebug>(flags: DebugFlags) -> Result {
        // prevent racing System init
        let _lock = GLOBAL_SYSTEM_STATE.read();
        // suppress default debug init
        DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

        fmod_try!(FMOD_Debug_Initialize(
            flags.into_raw(),
            DebugMode::Callback.into_raw(),
            Some(callback::<D>),
            ptr::null()
        ));
        Ok(())
    }

    /// Specify the level and delivery method of log messages when using the
    /// logging version of FMOD.
    ///
    /// This method initializes logs to go to write the log to the specified
    /// file path.
    ///
    /// This function will return [Error::Unsupported] when using the
    /// non-logging (release) versions of FMOD.
    ///
    /// The logging version of FMOD can be recognized by the 'L' suffix in the
    /// library name, fmodL.dll or libfmodL.so for instance.
    ///
    /// Note that:
    /// - [DebugFlags::LevelLog] produces informational, warning and error
    ///   messages.
    /// - [DebugFlags::LevelWarning] produces warnings and error messages.
    /// - [DebugFlags::LevelError] produces error messages only.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    /// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
    /// the <code>fmod_debug_is_tracing</code> feature flag is set. Manually
    /// initializing FMOD debugging will override this behavior.
    /// </pre>
    pub fn initialize_file(flags: DebugFlags, file: &CStr) -> Result {
        // prevent racing System init
        let _lock = GLOBAL_SYSTEM_STATE.read();
        // suppress default debug init
        DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

        fmod_try!(FMOD_Debug_Initialize(
            flags.into_raw(),
            DebugMode::File.into_raw(),
            None,
            file.as_ptr()
        ));
        Ok(())
    }

    #[cfg(feature = "tracing")]
    /// An `FmodDebug` sink that hooks up FMOD debug messages into [tracing].
    ///
    /// Mapping:
    /// - [DebugFlags::TypeMemory]   ~ [Level::TRACE], target `fmod::memory`
    /// - [DebugFlags::TypeFile]     ~ [Level::TRACE], target `fmod::file`
    /// - [DebugFlags::TypeCodec]    ~ [Level::TRACE], target `fmod::codec`
    /// - [DebugFlags::TypeTrace]    ~ [Level::TRACE], target `fmod::trace`
    /// - [DebugFlags::LevelLog]     ~ [Level::INFO],  target `fmod`
    /// - [DebugFlags::LevelWarning] ~ [Level::WARN],  target `fmod`
    /// - [DebugFlags::LevelError]   ~ [Level::ERROR], target `fmod`
    ///
    /// All events are emitted under a orphaned (no parent) error span with name
    /// and target `fmod` which is never explicitly entered nor exited.
    ///
    /// `file`, `line`, `func`, and `message` are fed as-is as event fields to
    /// the tracing backend.
    ///
    /// [Level::TRACE]: tracing::Level::TRACE
    /// [Level::DEBUG]: tracing::Level::DEBUG
    /// [Level::INFO]: tracing::Level::INFO
    /// [Level::WARN]: tracing::Level::WARN
    /// [Level::ERROR]: tracing::Level::ERROR
    pub enum FmodDebugTracing {}

    #[cfg(feature = "tracing")]
    impl FmodDebug for FmodDebugTracing {
        fn log(
            flags: DebugFlags,
            file: Option<&str>,
            line: i32,
            func: Option<&str>,
            message: Option<&str>,
        ) -> Result<()> {
            if flags.is_set(DebugFlags::TypeMemory) {
                tracing::trace!(target: "fmod::memory", parent: crate::span(), file, line, func, message)
            } else if flags.is_set(DebugFlags::TypeFile) {
                tracing::trace!(target: "fmod::file", parent: crate::span(), file, line, func, message)
            } else if flags.is_set(DebugFlags::TypeCodec) {
                tracing::trace!(target: "fmod::codec", parent: crate::span(), file, line, func, message)
            } else if flags.is_set(DebugFlags::TypeTrace) {
                tracing::trace!(target: "fmod::trace", parent: crate::span(), file, line, func, message)
            } else if flags.is_set(DebugFlags::LevelLog) {
                tracing::info!(target: "fmod", parent: crate::span(), file, line, func, message)
            } else if flags.is_set(DebugFlags::LevelWarning) {
                tracing::warn!(target: "fmod", parent: crate::span(), file, line, func, message)
            } else if flags.is_set(DebugFlags::LevelError) {
                tracing::error!(target: "fmod", parent: crate::span(), file, line, func, message)
            } else {
                tracing::error!(
                    parent: crate::span(),
                    debug_flags = ?flags,
                    file,
                    line,
                    func,
                    message,
                );
                return Err(Error::InternalRs);
            };
            Ok(())
        }
    }

    #[cfg(feature = "tracing")]
    impl FmodDebugTracing {
        /// Create [DebugFlags] enabling only debug logging which is enabled by
        /// the current default tracing subscriber.
        ///
        /// This relies on the current contextual subscriber being the actual
        /// subscriber which we eventually dispatch to; if the contextual
        /// subscriber is later changed to a subscriber that enables more than
        /// the original did, FMOD will not be emitting the events which the new
        /// subscriber but not the old subscriber are interested in.
        ///
        /// Filtering out logging on the FMOD side is a desirable default, but
        /// if you change the contextual subscriber, you should reinitialize the
        /// FMOD debug layer or just originally initialize it with
        /// DebugFlags::All and let dynamic filtering take care of it.
        ///
        /// FMOD's default filter is [DebugFlags::LevelLog], which is equivalent
        /// to a tracing filter of `fmod=INFO` when using `FmodDebugTracing`.
        /// Enabling the `fmod::memory`/`fmod::file`/`fmod::codec` targets at
        /// trace level should only be done when debugging specific issues that
        /// require tracing that area's execution; these are truly verbose trace
        /// level logging targets.
        pub fn ideal_debug_flags() -> DebugFlags {
            use tracing::{enabled, Level};
            let mut debug_flags = DebugFlags::LevelNone;

            if enabled!(target: "fmod", Level::ERROR, { file, line, func, message }) {
                debug_flags = DebugFlags::LevelError;
            }
            if enabled!(target: "fmod", Level::WARN, { file, line, func, message }) {
                debug_flags = DebugFlags::LevelWarning;
            }
            if enabled!(target: "fmod", Level::INFO, { file, line, func, message }) {
                debug_flags = DebugFlags::LevelLog;
            }
            if enabled!(target: "fmod::trace", Level::TRACE, { file, line, func, message }) {
                debug_flags |= DebugFlags::TypeTrace;
            }
            if enabled!(target: "fmod::memory", Level::TRACE, { file, line, func, message }) {
                debug_flags |= DebugFlags::TypeMemory;
            }
            if enabled!(target: "fmod::file", Level::TRACE, { file, line, func, message }) {
                debug_flags |= DebugFlags::TypeFile;
            }
            if enabled!(target: "fmod::codec", Level::TRACE, { file, line, func, message }) {
                debug_flags |= DebugFlags::TypeCodec;
            }

            debug_flags
        }
    }

    #[cfg(feature = "tracing")]
    fn handle_init_failure(error: Error) {
        match error {
            Error::Unsupported => {
                tracing::info!(parent: crate::span(), "FMOD debug disabled");
            },
            error => {
                tracing::error!(
                    parent: crate::span(),
                    "Error during FMOD debug initialization: {error}",
                )
            },
        }
    }

    #[cfg(feature = "tracing")]
    /// Initialize the FMOD debug layer to write to the contextual tracing
    /// subscriber via [`FmodDebugTracing`].
    ///
    /// If FMOD debugging is statically disabled, logs this via `info!`.
    ///
    /// If the `fmod_debug_is_tracing` feature flag is set, this is done
    /// automatially for you when an FMOD System is first created. This is only
    /// required when changing the contextual subscriber (see
    /// [`FmodDebugTracing::ideal_debug_flags`] for why) or if changing the
    /// FMOD debug layer's state at runtime.
    pub fn initialize_tracing() {
        match initialize_callback::<FmodDebugTracing>(FmodDebugTracing::ideal_debug_flags()) {
            Ok(()) => (),
            Err(error) => handle_init_failure(error),
        }
    }

    /// Initialize the FMOD debug layer to tracing only if it hasn't been
    /// already initialized and skipping the global read lock against racing
    /// system init. Called as part of system init.
    pub(crate) unsafe fn initialize_default() {
        #[cfg(feature = "fmod_debug_is_tracing")]
        {
            if DEBUG_LAYER_INITIALIZED
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                let flags = FmodDebugTracing::ideal_debug_flags();
                let error = FMOD_Debug_Initialize(
                    flags.into_raw(),
                    DebugMode::Callback.into_raw(),
                    Some(callback::<FmodDebugTracing>),
                    ptr::null(),
                );
                match Error::from_raw(error) {
                    None => (),
                    Some(error) => handle_init_failure(error),
                }
            }
        }
    }
}

pub mod file {
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
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        parent: crate::span(),
                        error = error.into_raw(),
                        "Error unlocking file busy state: {error}",
                    );
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
        fn open(name: &CStr, size: u32, handle: usize);
        /// Callback for after a file is closed.
        fn close(handle: usize);
        /// Callback for after a read operation.
        fn read(handle: usize, buffer: &[u8], eof: bool);
        /// Callback for after a seek operation.
        fn seek(handle: usize, pos: u32);
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
}

pub mod thread {
    use fmod::{raw::*, *};

    /// Specify the affinity, priority and stack size for all FMOD created
    /// threads.
    ///
    /// Affinity can be specified using one (or more) of the [ThreadAffinity]
    /// constants or by providing the bits explicitly, i.e. (1<<3) for logical
    /// core three (core affinity is zero based).  
    /// See platform documentation for details on the available cores for a
    /// given device.
    ///
    /// Priority can be specified using one of the [ThreadPriority] constants or
    /// by providing the value explicitly, i.e. (-2) for the lowest thread
    /// priority on Windows.  
    /// See platform documentation for details on the available priority values
    /// for a given operating system.
    ///
    /// Stack size can be specified explicitly, however for each thread you
    /// should provide a size equal to or larger than the expected default or
    /// risk causing a stack overflow at runtime.
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    pub unsafe fn set_attributes(
        kind: ThreadType,
        affinity: ThreadAffinity,
        priority: ThreadPriority,
        stack_size: ThreadStackSize,
    ) -> Result {
        fmod_try!(FMOD_Thread_SetAttributes(
            kind.into_raw(),
            affinity.into_raw(),
            priority.into_raw(),
            stack_size.into_raw()
        ));
        Ok(())
    }
}
