#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

include!("../inc/bindings.rs");

// fsbank_errors.h `static const char *FMOD_ErrorString(FMOD_RESULT result)`
#[inline]
#[allow(clippy::missing_safety_doc)]
pub const unsafe fn FSBank_ErrorString(result: FSBANK_RESULT) -> *const std::ffi::c_char {
    match result {
        FSBANK_OK =>                                    "No errors.\0",
        FSBANK_ERR_CACHE_CHUNKNOTFOUND =>               "An expected chunk is missing from the cache, perhaps try deleting cache files.\0",
        FSBANK_ERR_CANCELLED =>                         "The build process was cancelled during compilation by the user.\0",
        FSBANK_ERR_CANNOT_CONTINUE =>                   "The build process cannot continue due to previously ignored errors.\0",
        FSBANK_ERR_ENCODER =>                           "Encoder for chosen format has encountered an unexpected error.\0",
        FSBANK_ERR_ENCODER_INIT =>                      "Encoder initialization failed.\0",
        FSBANK_ERR_ENCODER_NOTSUPPORTED =>              "Encoder for chosen format is not supported on this platform.\0",
        FSBANK_ERR_FILE_OS =>                           "An operating system based file error was encountered.\0",
        FSBANK_ERR_FILE_NOTFOUND =>                     "A specified file could not be found.\0",
        FSBANK_ERR_FMOD =>                              "Internal error from FMOD sub-system.\0",
        FSBANK_ERR_INITIALIZED =>                       "Already initialized.\0",
        FSBANK_ERR_INVALID_FORMAT =>                    "The format of the source file is invalid.\0",
        FSBANK_ERR_INVALID_PARAM =>                     "An invalid parameter has been passed to this function.\0",
        FSBANK_ERR_MEMORY =>                            "Run out of memory.\0",
        FSBANK_ERR_UNINITIALIZED =>                     "Not initialized yet.\0",
        FSBANK_ERR_WRITER_FORMAT =>                     "Chosen encode format is not supported by this FSB version.\0",
        FSBANK_WARN_CANNOTLOOP =>                       "Source file is too short for seamless looping. Looping disabled.\0",
        FSBANK_WARN_IGNORED_FILTERHIGHFREQ =>           "FSBANK_BUILD_FILTERHIGHFREQ flag ignored: feature only supported by XMA format.\0",
        FSBANK_WARN_IGNORED_DISABLESEEKING =>           "FSBANK_BUILD_DISABLESEEKING flag ignored: feature only supported by XMA format.\0",
        FSBANK_WARN_FORCED_DONTWRITENAMES =>            "FSBANK_BUILD_FSB5_DONTWRITENAMES flag forced: cannot write names when source is from memory.\0",
        FSBANK_ERR_ENCODER_FILE_NOTFOUND =>             "External encoder dynamic library not found.\0",
        FSBANK_ERR_ENCODER_FILE_BAD =>                  "External encoder dynamic library could not be loaded, possibly incorrect binary format, incorrect architecture, or file corruption.\0",
        _ =>                                            "Unknown error.\0",
    }.as_ptr() as _
}
