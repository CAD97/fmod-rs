/* automatically @generated by rust-bindgen 0.69.4 */
/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2024. */

pub type FSBANK_INITFLAGS = ::std::ffi::c_uint;
pub type FSBANK_BUILDFLAGS = ::std::ffi::c_uint;
pub type FSBANK_RESULT = ::std::ffi::c_int;
pub type FSBANK_FORMAT = ::std::ffi::c_int;
pub type FSBANK_FSBVERSION = ::std::ffi::c_int;
pub type FSBANK_STATE = ::std::ffi::c_int;
pub type FSBANK_MEMORY_ALLOC_CALLBACK = ::std::option::Option<
    unsafe extern "system" fn(
        size: ::std::ffi::c_uint,
        r#type: ::std::ffi::c_uint,
        sourceStr: *const ::std::ffi::c_char,
    ) -> *mut ::std::ffi::c_void,
>;
pub type FSBANK_MEMORY_REALLOC_CALLBACK = ::std::option::Option<
    unsafe extern "system" fn(
        ptr: *mut ::std::ffi::c_void,
        size: ::std::ffi::c_uint,
        r#type: ::std::ffi::c_uint,
        sourceStr: *const ::std::ffi::c_char,
    ) -> *mut ::std::ffi::c_void,
>;
pub type FSBANK_MEMORY_FREE_CALLBACK = ::std::option::Option<
    unsafe extern "system" fn(
        ptr: *mut ::std::ffi::c_void,
        r#type: ::std::ffi::c_uint,
        sourceStr: *const ::std::ffi::c_char,
    ),
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_SUBSOUND {
    pub fileNames: *const *const ::std::ffi::c_char,
    pub fileData: *const *const ::std::ffi::c_void,
    pub fileDataLengths: *const ::std::ffi::c_uint,
    pub numFiles: ::std::ffi::c_uint,
    pub overrideFlags: FSBANK_BUILDFLAGS,
    pub overrideQuality: ::std::ffi::c_uint,
    pub desiredSampleRate: ::std::ffi::c_float,
    pub percentOptimizedRate: ::std::ffi::c_float,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_PROGRESSITEM {
    pub subSoundIndex: ::std::ffi::c_int,
    pub threadIndex: ::std::ffi::c_int,
    pub state: FSBANK_STATE,
    pub stateData: *const ::std::ffi::c_void,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_STATEDATA_FAILED {
    pub errorCode: FSBANK_RESULT,
    pub errorString: [::std::ffi::c_char; 256usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FSBANK_STATEDATA_WARNING {
    pub warnCode: FSBANK_RESULT,
    pub warningString: [::std::ffi::c_char; 256usize],
}
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
pub const FSBANK_BUILD_ALIGN4K: u32 = 1024;
pub const FSBANK_BUILD_OVERRIDE_MASK: u32 = 543;
pub const FSBANK_BUILD_CACHE_VALIDATION_MASK: u32 = 22;
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
pub const FSBANK_WARN_IGNORED_ALIGN4K: FSBANK_RESULT = 22;
pub const FSBANK_FORMAT_PCM: FSBANK_FORMAT = 0;
pub const FSBANK_FORMAT_XMA: FSBANK_FORMAT = 1;
pub const FSBANK_FORMAT_AT9: FSBANK_FORMAT = 2;
pub const FSBANK_FORMAT_VORBIS: FSBANK_FORMAT = 3;
pub const FSBANK_FORMAT_FADPCM: FSBANK_FORMAT = 4;
pub const FSBANK_FORMAT_OPUS: FSBANK_FORMAT = 5;
pub const FSBANK_FORMAT_MAX: FSBANK_FORMAT = 6;
pub const FSBANK_FSBVERSION_FSB5: FSBANK_FSBVERSION = 0;
pub const FSBANK_FSBVERSION_MAX: FSBANK_FSBVERSION = 1;
pub const FSBANK_STATE_DECODING: FSBANK_STATE = 0;
pub const FSBANK_STATE_ANALYSING: FSBANK_STATE = 1;
pub const FSBANK_STATE_PREPROCESSING: FSBANK_STATE = 2;
pub const FSBANK_STATE_ENCODING: FSBANK_STATE = 3;
pub const FSBANK_STATE_WRITING: FSBANK_STATE = 4;
pub const FSBANK_STATE_FINISHED: FSBANK_STATE = 5;
pub const FSBANK_STATE_FAILED: FSBANK_STATE = 6;
pub const FSBANK_STATE_WARNING: FSBANK_STATE = 7;
extern "system" {
    pub fn FSBank_MemoryInit(
        userAlloc: FSBANK_MEMORY_ALLOC_CALLBACK,
        userRealloc: FSBANK_MEMORY_REALLOC_CALLBACK,
        userFree: FSBANK_MEMORY_FREE_CALLBACK,
    ) -> FSBANK_RESULT;
    pub fn FSBank_Init(
        version: FSBANK_FSBVERSION,
        flags: FSBANK_INITFLAGS,
        numSimultaneousJobs: ::std::ffi::c_uint,
        cacheDirectory: *const ::std::ffi::c_char,
    ) -> FSBANK_RESULT;
    pub fn FSBank_Release() -> FSBANK_RESULT;
    pub fn FSBank_Build(
        subSounds: *const FSBANK_SUBSOUND,
        numSubSounds: ::std::ffi::c_uint,
        encodeFormat: FSBANK_FORMAT,
        buildFlags: FSBANK_BUILDFLAGS,
        quality: ::std::ffi::c_uint,
        encryptKey: *const ::std::ffi::c_char,
        outputFileName: *const ::std::ffi::c_char,
    ) -> FSBANK_RESULT;
    pub fn FSBank_FetchFSBMemory(
        data: *mut *const ::std::ffi::c_void,
        length: *mut ::std::ffi::c_uint,
    ) -> FSBANK_RESULT;
    pub fn FSBank_BuildCancel() -> FSBANK_RESULT;
    pub fn FSBank_FetchNextProgressItem(
        progressItem: *mut *const FSBANK_PROGRESSITEM,
    ) -> FSBANK_RESULT;
    pub fn FSBank_ReleaseProgressItem(progressItem: *const FSBANK_PROGRESSITEM) -> FSBANK_RESULT;
    pub fn FSBank_MemoryGetStats(
        currentAllocated: *mut ::std::ffi::c_uint,
        maximumAllocated: *mut ::std::ffi::c_uint,
    ) -> FSBANK_RESULT;
}
