pub mod memory {
    #[cfg(doc)]
    use fmod::*;
    use {
        fmod::{raw::*, MemoryType, Result, GLOBAL_SYSTEM_STATE},
        std::{
            alloc::{alloc, dealloc, realloc, Layout},
            mem::{self, MaybeUninit},
            os::raw::{c_char, c_void},
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
    /// memory. To specify a custom memory management strategy, use
    /// [`memory::initialize_pool`], [`memory::initialize_alloc`], or
    /// [`memory::initialize_rust_global_alloc`].
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

    /// Specifies for FMOD to allocate and free memory in a user supplied memory
    /// buffer with a fixed size.
    ///
    /// The buffer is truncated to a multiple of 512 bytes.
    ///
    /// To find out the required fixed size call [memory::initialize_pool] with
    /// an overly large pool size (or no pool) and find out the maximum RAM
    /// usage at any one time with [memory::get_stats].
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

    /// Callback to allocate a block of memory.
    ///
    /// Returning an aligned pointer, of 16 byte alignment is recommended for
    /// performance reasons.
    ///
    /// <dl>
    /// <dt>size</dt><dd>Size in bytes of the memory block to be allocated and
    /// returned.</dd>
    /// <dt>kind</dt><dd>Type of memory allocation.</dd>
    /// <dt>source</dt><dd>String with the FMOD source code filename and line
    /// number in it. Only valid in logging versions of FMOD.</dd>
    /// </dl>
    pub type Alloc =
        unsafe extern "C" fn(size: u32, kind: MemoryType, source: *const c_char) -> *mut c_void;

    /// Callback to re-allocate a block of memory to a different size.
    ///
    /// When allocating new memory, the contents of the old memory block must be
    /// preserved.
    ///
    /// Returning an aligned pointer, of 16 byte alignment is recommended for
    /// performance reasons.
    ///
    /// <dl>
    /// <dt>ptr</dt><dd>Block of memory to be resized. If this is null, then a
    /// new block of memory is allocated and no memory is freed.</dd>
    /// <dt>size</dt><dd>Size in bytes of the memory block to be reallocated.</dd>
    /// <dt>kind</dt><dd>Memory allocation type.</dd>
    /// <dt>source</dt><dd>String with the FMOD source code filename and line
    /// number in it. Only valid in logging versions of FMOD.</dd>
    /// </dl>
    pub type Realloc = unsafe extern "C" fn(
        ptr: *mut c_void,
        size: u32,
        kind: MemoryType,
        source: *const c_char,
    ) -> *mut c_void;

    /// Callback to free a block of memory.
    ///
    /// <dl>
    /// <dt>ptr</dt><dd>Pre-existing block of memory to be freed.</dd>
    /// <dt>kind</dt><dd>Type of memory to be freed.</dd>
    /// <dt>source</dt><dd>String with the FMOD source code filename and line
    /// number in it. Only valid in logging versions of FMOD.</dd>
    /// </dl>
    pub type Free = unsafe extern "C" fn(ptr: *mut c_void, kind: MemoryType, source: *const c_char);

    /// Specifies for FMOD to allocate and free memory through user supplied
    /// callbacks.
    ///
    /// Callback implementations must be thread safe. Callbacks will be used
    /// only for memory types specified by the `mem_type_flags` parameter.
    ///
    /// If `alloc` and `free` are provided without `realloc` the reallocation is
    /// implemented via an allocation of the new size, copy from old address to
    /// new, then a free of the old address.
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    pub unsafe fn initialize_alloc(
        alloc: Alloc,
        realloc: Option<Realloc>,
        free: Free,
        mem_type_flags: MemoryType,
    ) -> Result {
        fmod_try!(FMOD_Memory_Initialize(
            ptr::null_mut(),
            0,
            mem::transmute(alloc),
            mem::transmute(realloc),
            mem::transmute(free),
            mem_type_flags.into_raw(),
        ));
        Ok(())
    }

    /// Specifies for FMOD to allocate and free memory via the Rust global
    /// allocator. *This has overhead:* FMOD expects an ANSI `malloc`/`free`
    /// interface without a size parameter to `free`. Rust's global allocator
    /// requires `Layout` to be given on `dealloc`, so filling an ANSI `free`
    /// shaped hole with Rust's `dealloc` requires storing size information
    /// somewhere.
    ///
    /// This function takes the traditional approach used by allocators and
    /// over-allocates from what was requested to store the layout information
    /// in the allocation. Unfortunately, since most general purpose allocators
    /// have an ANSI-compatible interface, this is purely a waste in most
    /// situations, requried just to satisfy the Rust middleman.
    ///
    /// In most cases, you should prefer leaving the defaults (FMOD will use the
    /// system allocator) or one of the other [`memory::initialize`] variants
    /// which don't require this overhead.
    ///
    /// # Safety
    ///
    /// This function must be called before any FMOD [System] object is created.
    #[cfg(feature = "rust_alloc_bridge")]
    pub unsafe fn initialize_rust_global_alloc() -> Result {
        const _: u8 = [0u8; 17][mem::size_of::<Layout>()];

        unsafe extern "C" fn fmod_rust_alloc(
            size: u32,
            _: MemoryType,
            _: *const c_char,
        ) -> *mut c_void {
            let layout = Layout::from_size_align_unchecked(size as usize + 16, 16);
            let ptr = alloc(layout);
            if ptr.is_null() {
                return ptr::null_mut(); // let FMOD handle it, it'll be *fine*
            }
            ptr.cast::<Layout>().write(layout);
            ptr.add(16).cast()
        }

        unsafe extern "C" fn fmod_rust_realloc(
            ptr: *mut c_void,
            size: u32,
            _: MemoryType,
            _: *const c_char,
        ) -> *mut c_void {
            let new_layout = Layout::from_size_align_unchecked(size as usize + 16, 16);
            let ptr = ptr.cast::<u8>().sub(16);
            let old_layout = ptr.cast::<Layout>().read();
            let ptr = realloc(ptr, old_layout, new_layout.size());
            if ptr.is_null() {
                return ptr::null_mut(); // let FMOD handle it, it'll be *fine*
            }
            ptr.cast::<Layout>().write(new_layout);
            ptr.add(16).cast()
        }

        unsafe extern "C" fn fmod_rust_free(ptr: *mut c_void, _: MemoryType, _: *const c_char) {
            let ptr = ptr.cast::<u8>().sub(16);
            let layout = ptr.cast::<Layout>().read();
            dealloc(ptr, layout)
        }

        initialize_alloc(
            fmod_rust_alloc,
            Some(fmod_rust_realloc),
            fmod_rust_free,
            MemoryType::All,
        )
    }
}

pub mod debug {
    use fmod::{Error, Result};

    /// Specify the level and delivery method of log messages when using the
    /// logging version of FMOD.
    ///
    /// This function will return [Error::Unsupported] when using the
    /// non-logging (release) versions of FMOD.
    ///
    /// <pre class="ignore" style="white-space:normal;font:inherit;">
    /// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
    /// the <code>fmod_debug_is_tracing</code> flag is set. It is not currently
    /// supported to override this behavior and manually initialize FMOD's
    /// debugging.
    /// </pre>
    pub fn initialize() -> Result {
        Err(Error::Unsupported)
    }
}

pub mod file {
    #[cfg(doc)]
    use fmod::*;
    use fmod::{raw::*, Result, GLOBAL_SYSTEM_STATE};

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
}

pub mod thread {
    #[cfg(doc)]
    use fmod::*;
    use fmod::{raw::*, Result, ThreadAffinity, ThreadPriority, ThreadStackSize, ThreadType};

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
