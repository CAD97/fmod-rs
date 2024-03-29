/* automatically generated by rust-bindgen 0.69.1 */

/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2023. */

pub const FSBANK_INIT_NORMAL: u32 = 0;
pub const FSBANK_INIT_IGNOREERRORS: u32 = 1;
pub const FSBANK_INIT_WARNINGSASERRORS: u32 = 2;
pub const FSBANK_INIT_CREATEINCLUDEHEADER: u32 = 4;
pub const FSBANK_INIT_DONTLOADCACHEFILES: u32 = 8;
pub const FSBANK_INIT_GENERATEPROGRESSITEMS: u32 = 16;
pub const FSBANK_BUILD_DEFAULT: u32 = 0;
pub const FSBANK_BUILD_DISABLESYNCPOINTS: u32 = 1;
pub const FSBANK_BUILD_DONTLOOP: u32 = 2;
pub const FSBANK_BUILD_FILTERHIGHFREQ: u32 = 4;
pub const FSBANK_BUILD_DISABLESEEKING: u32 = 8;
pub const FSBANK_BUILD_OPTIMIZESAMPLERATE: u32 = 16;
pub const FSBANK_BUILD_FSB5_DONTWRITENAMES: u32 = 128;
pub const FSBANK_BUILD_NOGUID: u32 = 256;
pub const FSBANK_BUILD_WRITEPEAKVOLUME: u32 = 512;
pub const FSBANK_BUILD_OVERRIDE_MASK: u32 = 543;
pub const FSBANK_BUILD_CACHE_VALIDATION_MASK: u32 = 22;
pub type FSBANK_INITFLAGS = ::std::os::raw::c_uint;
pub type FSBANK_BUILDFLAGS = ::std::os::raw::c_uint;
pub const FSBANK_OK: FSBANK_RESULT = 0;
pub const FSBANK_ERR_CACHE_CHUNKNOTFOUND: FSBANK_RESULT = 1;
pub const FSBANK_ERR_CANCELLED: FSBANK_RESULT = 2;
pub const FSBANK_ERR_CANNOT_CONTINUE: FSBANK_RESULT = 3;
pub const FSBANK_ERR_ENCODER: FSBANK_RESULT = 4;
pub const FSBANK_ERR_ENCODER_INIT: FSBANK_RESULT = 5;
pub const FSBANK_ERR_ENCODER_NOTSUPPORTED: FSBANK_RESULT = 6;
pub const FSBANK_ERR_FILE_OS: FSBANK_RESULT = 7;
pub const FSBANK_ERR_FILE_NOTFOUND: FSBANK_RESULT = 8;
pub const FSBANK_ERR_FMOD: FSBANK_RESULT = 9;
pub const FSBANK_ERR_INITIALIZED: FSBANK_RESULT = 10;
pub const FSBANK_ERR_INVALID_FORMAT: FSBANK_RESULT = 11;
pub const FSBANK_ERR_INVALID_PARAM: FSBANK_RESULT = 12;
pub const FSBANK_ERR_MEMORY: FSBANK_RESULT = 13;
pub const FSBANK_ERR_UNINITIALIZED: FSBANK_RESULT = 14;
pub const FSBANK_ERR_WRITER_FORMAT: FSBANK_RESULT = 15;
pub const FSBANK_WARN_CANNOTLOOP: FSBANK_RESULT = 16;
pub const FSBANK_WARN_IGNORED_FILTERHIGHFREQ: FSBANK_RESULT = 17;
pub const FSBANK_WARN_IGNORED_DISABLESEEKING: FSBANK_RESULT = 18;
pub const FSBANK_WARN_FORCED_DONTWRITENAMES: FSBANK_RESULT = 19;
pub const FSBANK_ERR_ENCODER_FILE_NOTFOUND: FSBANK_RESULT = 20;
pub const FSBANK_ERR_ENCODER_FILE_BAD: FSBANK_RESULT = 21;
pub type FSBANK_RESULT = ::std::os::raw::c_int;
pub const FSBANK_FORMAT_PCM: FSBANK_FORMAT = 0;
pub const FSBANK_FORMAT_XMA: FSBANK_FORMAT = 1;
pub const FSBANK_FORMAT_AT9: FSBANK_FORMAT = 2;
pub const FSBANK_FORMAT_VORBIS: FSBANK_FORMAT = 3;
pub const FSBANK_FORMAT_FADPCM: FSBANK_FORMAT = 4;
pub const FSBANK_FORMAT_OPUS: FSBANK_FORMAT = 5;
pub const FSBANK_FORMAT_MAX: FSBANK_FORMAT = 6;
pub type FSBANK_FORMAT = ::std::os::raw::c_int;
pub const FSBANK_FSBVERSION_FSB5: FSBANK_FSBVERSION = 0;
pub const FSBANK_FSBVERSION_MAX: FSBANK_FSBVERSION = 1;
pub type FSBANK_FSBVERSION = ::std::os::raw::c_int;
pub const FSBANK_STATE_DECODING: FSBANK_STATE = 0;
pub const FSBANK_STATE_ANALYSING: FSBANK_STATE = 1;
pub const FSBANK_STATE_PREPROCESSING: FSBANK_STATE = 2;
pub const FSBANK_STATE_ENCODING: FSBANK_STATE = 3;
pub const FSBANK_STATE_WRITING: FSBANK_STATE = 4;
pub const FSBANK_STATE_FINISHED: FSBANK_STATE = 5;
pub const FSBANK_STATE_FAILED: FSBANK_STATE = 6;
pub const FSBANK_STATE_WARNING: FSBANK_STATE = 7;
pub type FSBANK_STATE = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_SUBSOUND {
    pub fileNames: *const *const ::std::os::raw::c_char,
    pub fileData: *const *const ::std::os::raw::c_void,
    pub fileDataLengths: *const ::std::os::raw::c_uint,
    pub numFiles: ::std::os::raw::c_uint,
    pub overrideFlags: FSBANK_BUILDFLAGS,
    pub overrideQuality: ::std::os::raw::c_uint,
    pub desiredSampleRate: f32,
    pub percentOptimizedRate: f32,
}
#[test]
fn bindgen_test_layout_FSBANK_SUBSOUND() {
    const UNINIT: ::std::mem::MaybeUninit<FSBANK_SUBSOUND> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<FSBANK_SUBSOUND>(),
        48usize,
        concat!("Size of: ", stringify!(FSBANK_SUBSOUND))
    );
    assert_eq!(
        ::std::mem::align_of::<FSBANK_SUBSOUND>(),
        8usize,
        concat!("Alignment of ", stringify!(FSBANK_SUBSOUND))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fileNames) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(fileNames)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fileData) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(fileData)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).fileDataLengths) as usize - ptr as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(fileDataLengths)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).numFiles) as usize - ptr as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(numFiles)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).overrideFlags) as usize - ptr as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(overrideFlags)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).overrideQuality) as usize - ptr as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(overrideQuality)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).desiredSampleRate) as usize - ptr as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(desiredSampleRate)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).percentOptimizedRate) as usize - ptr as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_SUBSOUND),
            "::",
            stringify!(percentOptimizedRate)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_PROGRESSITEM {
    pub subSoundIndex: ::std::os::raw::c_int,
    pub threadIndex: ::std::os::raw::c_int,
    pub state: FSBANK_STATE,
    pub stateData: *const ::std::os::raw::c_void,
}
#[test]
fn bindgen_test_layout_FSBANK_PROGRESSITEM() {
    const UNINIT: ::std::mem::MaybeUninit<FSBANK_PROGRESSITEM> = ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<FSBANK_PROGRESSITEM>(),
        24usize,
        concat!("Size of: ", stringify!(FSBANK_PROGRESSITEM))
    );
    assert_eq!(
        ::std::mem::align_of::<FSBANK_PROGRESSITEM>(),
        8usize,
        concat!("Alignment of ", stringify!(FSBANK_PROGRESSITEM))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).subSoundIndex) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_PROGRESSITEM),
            "::",
            stringify!(subSoundIndex)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).threadIndex) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_PROGRESSITEM),
            "::",
            stringify!(threadIndex)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).state) as usize - ptr as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_PROGRESSITEM),
            "::",
            stringify!(state)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).stateData) as usize - ptr as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_PROGRESSITEM),
            "::",
            stringify!(stateData)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_STATEDATA_FAILED {
    pub errorCode: FSBANK_RESULT,
    pub errorString: [::std::os::raw::c_char; 256usize],
}
#[test]
fn bindgen_test_layout_FSBANK_STATEDATA_FAILED() {
    const UNINIT: ::std::mem::MaybeUninit<FSBANK_STATEDATA_FAILED> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<FSBANK_STATEDATA_FAILED>(),
        260usize,
        concat!("Size of: ", stringify!(FSBANK_STATEDATA_FAILED))
    );
    assert_eq!(
        ::std::mem::align_of::<FSBANK_STATEDATA_FAILED>(),
        4usize,
        concat!("Alignment of ", stringify!(FSBANK_STATEDATA_FAILED))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).errorCode) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_STATEDATA_FAILED),
            "::",
            stringify!(errorCode)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).errorString) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_STATEDATA_FAILED),
            "::",
            stringify!(errorString)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_STATEDATA_WARNING {
    pub warnCode: FSBANK_RESULT,
    pub warningString: [::std::os::raw::c_char; 256usize],
}
#[test]
fn bindgen_test_layout_FSBANK_STATEDATA_WARNING() {
    const UNINIT: ::std::mem::MaybeUninit<FSBANK_STATEDATA_WARNING> =
        ::std::mem::MaybeUninit::uninit();
    let ptr = UNINIT.as_ptr();
    assert_eq!(
        ::std::mem::size_of::<FSBANK_STATEDATA_WARNING>(),
        260usize,
        concat!("Size of: ", stringify!(FSBANK_STATEDATA_WARNING))
    );
    assert_eq!(
        ::std::mem::align_of::<FSBANK_STATEDATA_WARNING>(),
        4usize,
        concat!("Alignment of ", stringify!(FSBANK_STATEDATA_WARNING))
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).warnCode) as usize - ptr as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_STATEDATA_WARNING),
            "::",
            stringify!(warnCode)
        )
    );
    assert_eq!(
        unsafe { ::std::ptr::addr_of!((*ptr).warningString) as usize - ptr as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(FSBANK_STATEDATA_WARNING),
            "::",
            stringify!(warningString)
        )
    );
}
pub type FSBANK_MEMORY_ALLOC_CALLBACK = ::std::option::Option<
    unsafe extern "system" fn(
        size: ::std::os::raw::c_uint,
        r#type: ::std::os::raw::c_uint,
        sourceStr: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_void,
>;
pub type FSBANK_MEMORY_REALLOC_CALLBACK = ::std::option::Option<
    unsafe extern "system" fn(
        ptr: *mut ::std::os::raw::c_void,
        size: ::std::os::raw::c_uint,
        r#type: ::std::os::raw::c_uint,
        sourceStr: *const ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_void,
>;
pub type FSBANK_MEMORY_FREE_CALLBACK = ::std::option::Option<
    unsafe extern "system" fn(
        ptr: *mut ::std::os::raw::c_void,
        r#type: ::std::os::raw::c_uint,
        sourceStr: *const ::std::os::raw::c_char,
    ),
>;
extern "system" {
    pub fn FSBank_MemoryInit(
        userAlloc: FSBANK_MEMORY_ALLOC_CALLBACK,
        userRealloc: FSBANK_MEMORY_REALLOC_CALLBACK,
        userFree: FSBANK_MEMORY_FREE_CALLBACK,
    ) -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_Init(
        version: FSBANK_FSBVERSION,
        flags: FSBANK_INITFLAGS,
        numSimultaneousJobs: ::std::os::raw::c_uint,
        cacheDirectory: *const ::std::os::raw::c_char,
    ) -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_Release() -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_Build(
        subSounds: *const FSBANK_SUBSOUND,
        numSubSounds: ::std::os::raw::c_uint,
        encodeFormat: FSBANK_FORMAT,
        buildFlags: FSBANK_BUILDFLAGS,
        quality: ::std::os::raw::c_uint,
        encryptKey: *const ::std::os::raw::c_char,
        outputFileName: *const ::std::os::raw::c_char,
    ) -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_FetchFSBMemory(
        data: *mut *const ::std::os::raw::c_void,
        length: *mut ::std::os::raw::c_uint,
    ) -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_BuildCancel() -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_FetchNextProgressItem(
        progressItem: *mut *const FSBANK_PROGRESSITEM,
    ) -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_ReleaseProgressItem(progressItem: *const FSBANK_PROGRESSITEM) -> FSBANK_RESULT;
}
extern "system" {
    pub fn FSBank_MemoryGetStats(
        currentAllocated: *mut ::std::os::raw::c_uint,
        maximumAllocated: *mut ::std::os::raw::c_uint,
    ) -> FSBANK_RESULT;
}
