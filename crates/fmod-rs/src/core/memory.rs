//! Functionality relating to FMOD's memory management.

use {
    crate::utils::{catch_user_unwind, str_from_nonnull_unchecked},
    fmod::{raw::*, *},
    std::{
        alloc::{alloc, dealloc, realloc, Layout},
        ffi::{c_char, c_uint, c_void},
        mem::{self, MaybeUninit},
        ptr,
    },
};

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
pub fn get_stats(blocking: bool) -> Result<Stats> {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();

    let mut current_alloced = 0;
    let mut max_alloced = 0;
    ffi!(FMOD_Memory_GetStats(
        &mut current_alloced,
        &mut max_alloced,
        blocking as FMOD_BOOL,
    ))?;

    Ok(Stats {
        current_alloced,
        max_alloced,
    })
}

/// Information on the memory usage of FMOD.
#[derive(Debug)]
pub struct Stats {
    /// Currently allocated memory at time of call.
    pub current_alloced: i32,
    /// Maximum allocated memory since [System::init] or
    /// [memory::initialize].
    pub max_alloced: i32,
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
    ffi!(FMOD_Memory_Initialize(
        pool.as_mut_ptr().cast(),
        pool_len.try_into().unwrap_or(i32::MAX % 512),
        None,
        None,
        None,
        0
    ))?;
    Ok(())
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
pub unsafe fn initialize_alloc<A: AllocCallback>(mem_type_flags: MemoryType) -> Result {
    ffi!(FMOD_Memory_Initialize(
        ptr::null_mut(),
        0,
        Some(useralloc::<A>),
        None,
        Some(userfree::<A>),
        mem_type_flags.into_raw(),
    ))?;
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
pub unsafe fn initialize_realloc<A: ReallocCallback>(mem_type_flags: MemoryType) -> Result {
    ffi!(FMOD_Memory_Initialize(
        ptr::null_mut(),
        0,
        Some(useralloc::<A>),
        Some(userrealloc::<A>),
        Some(userfree::<A>),
        mem_type_flags.into_raw(),
    ))?;
    Ok(())
}

// -------------------------------------------------------------------------------------------------

/// User callbacks for FMOD to (re)allocate and free memory.
///
/// Callback implementations must be thread safe and must not unwind.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait AllocCallback {
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

unsafe extern "system" fn useralloc<A: AllocCallback>(
    size: c_uint,
    kind: FMOD_MEMORY_TYPE,
    source: *const c_char,
) -> *mut c_void {
    catch_user_unwind(|| {
        // SAFETY: these strings are usually produced directly by FMOD, so they
        // should *actually* be guaranteed to be UTF-8 like FMOD claims.
        // However, plugins can also call this function, so we can't be sure.
        // Because alloc is such a perf-critical path and plugins are developer
        // controlled, assuming UTF-8 is both necessary and probably fine.
        let source = ptr::NonNull::new(source as *mut _).map(|x| str_from_nonnull_unchecked(x));
        Ok(A::alloc(size, MemoryType::from_raw(kind), source).cast())
    })
    .unwrap_or(ptr::null_mut())
}

unsafe extern "system" fn userrealloc<A: ReallocCallback>(
    ptr: *mut c_void,
    size: c_uint,
    kind: FMOD_MEMORY_TYPE,
    source: *const c_char,
) -> *mut c_void {
    catch_user_unwind(|| {
        // SAFETY: these strings are usually produced directly by FMOD, so they
        // should *actually* be guaranteed to be UTF-8 like FMOD claims.
        // However, plugins can also call this function, so we can't be sure.
        // Because alloc is such a perf-critical path and plugins are developer
        // controlled, assuming UTF-8 is both necessary and probably fine.
        let source = ptr::NonNull::new(source as *mut _).map(|x| str_from_nonnull_unchecked(x));
        Ok(A::realloc(ptr.cast(), size, MemoryType::from_raw(kind), source).cast())
    })
    .unwrap_or(ptr::null_mut())
}

/// User callbacks for FMOD to allocate and free memory.
///
/// Callback implementations must be thread safe and must not unwind.
#[allow(clippy::missing_safety_doc)]
pub unsafe trait ReallocCallback: AllocCallback {
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
    unsafe fn realloc(ptr: *mut u8, size: u32, kind: MemoryType, source: Option<&str>) -> *mut u8;
}

unsafe extern "system" fn userfree<A: AllocCallback>(
    ptr: *mut c_void,
    kind: FMOD_MEMORY_TYPE,
    source: *const c_char,
) {
    catch_user_unwind(|| {
        // SAFETY: these strings are usually produced directly by FMOD, so they
        // should *actually* be guaranteed to be UTF-8 like FMOD claims.
        // However, plugins can also call this function, so we can't be sure.
        // Because alloc is such a perf-critical path and plugins are developer
        // controlled, assuming UTF-8 is both necessary and probably fine.
        let source = ptr::NonNull::new(source as *mut _).map(|x| str_from_nonnull_unchecked(x));
        Ok(A::free(ptr.cast(), MemoryType::from_raw(kind), source))
    })
    .unwrap_or_default()
}

// -------------------------------------------------------------------------------------------------

flags! {
    /// Bitfields for memory allocation type being passed into FMOD memory callbacks.
    pub struct MemoryType: FMOD_MEMORY_TYPE {
        #[default]
        /// Standard memory.
        Normal       = FMOD_MEMORY_NORMAL,
        /// Stream file buffer, size controllable with [System::set_stream_buffer_size].
        StreamFile   = FMOD_MEMORY_STREAM_FILE,
        /// Stream decode buffer, size controllable with [CreateSoundExInfo::decode_buffer_size].
        StreamDecode = FMOD_MEMORY_STREAM_DECODE,
        /// Sample data buffer. Raw audio data, usually PCM/MPEG/ADPCM/XMA data.
        SampleData   = FMOD_MEMORY_SAMPLEDATA,
        /// Deprecated.
        DspBuffer    = FMOD_MEMORY_DSP_BUFFER,
        /// Memory allocated by a third party plugin.
        Plugin       = FMOD_MEMORY_PLUGIN,
        /// Persistent memory. Memory will be freed when the system is freed.
        Persistent   = FMOD_MEMORY_PERSISTENT,
        /// Mask specifying all memory types.
        All          = FMOD_MEMORY_ALL,
    }
}

// -------------------------------------------------------------------------------------------------

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
/// situations, required just to satisfy the Rust middleman.
///
/// In most cases, you should prefer leaving the defaults (FMOD will use the
/// system allocator) or one of the other [`memory::initialize`] variants
/// which don't require this overhead.
pub enum AllocViaRust {}

unsafe impl AllocCallback for AllocViaRust {
    fn alloc(size: u32, _: MemoryType, _: Option<&str>) -> *mut u8 {
        unsafe {
            let layout = Layout::from_size_align_unchecked(ix!(size) + 16, 16).pad_to_align();
            let ptr = alloc(layout);
            if ptr.is_null() {
                return ptr;
            }

            static_assert!(mem::size_of::<usize>() <= 16);
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

unsafe impl ReallocCallback for AllocViaRust {
    unsafe fn realloc(ptr: *mut u8, size: u32, _: MemoryType, _: Option<&str>) -> *mut u8 {
        let ptr = ptr.sub(16);
        let old_size = ptr.cast::<usize>().read();
        let old_layout = Layout::from_size_align_unchecked(old_size, 16);

        let new_layout = Layout::from_size_align_unchecked(ix!(size) + 16, 16).pad_to_align();
        let ptr = realloc(ptr, old_layout, new_layout.size());
        if ptr.is_null() {
            return ptr;
        }

        ptr.cast::<usize>().write(new_layout.size());
        ptr.add(16)
    }
}
