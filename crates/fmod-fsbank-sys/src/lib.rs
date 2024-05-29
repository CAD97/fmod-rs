#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

include!("../inc/bindings.rs");

// fsbank_errors.h `static const char *FMOD_ErrorString(FMOD_RESULT result)`
#[inline]
#[allow(clippy::missing_safety_doc)]
pub const unsafe fn FSBank_ErrorString(result: FSBANK_RESULT) -> *const std::ffi::c_char {
    match result {
        FSBANK_OK =>                                    c"No errors.",
        FSBANK_ERR_CACHE_CHUNKNOTFOUND =>               c"An expected chunk is missing from the cache, perhaps try deleting cache files.",
        FSBANK_ERR_CANCELLED =>                         c"The build process was cancelled during compilation by the user.",
        FSBANK_ERR_CANNOT_CONTINUE =>                   c"The build process cannot continue due to previously ignored errors.",
        FSBANK_ERR_ENCODER =>                           c"Encoder for chosen format has encountered an unexpected error.",
        FSBANK_ERR_ENCODER_INIT =>                      c"Encoder initialization failed.",
        FSBANK_ERR_ENCODER_NOTSUPPORTED =>              c"Encoder for chosen format is not supported on this platform.",
        FSBANK_ERR_FILE_OS =>                           c"An operating system based file error was encountered.",
        FSBANK_ERR_FILE_NOTFOUND =>                     c"A specified file could not be found.",
        FSBANK_ERR_FMOD =>                              c"Internal error from FMOD sub-system.",
        FSBANK_ERR_INITIALIZED =>                       c"Already initialized.",
        FSBANK_ERR_INVALID_FORMAT =>                    c"The format of the source file is invalid.",
        FSBANK_ERR_INVALID_PARAM =>                     c"An invalid parameter has been passed to this function.",
        FSBANK_ERR_MEMORY =>                            c"Run out of memory.",
        FSBANK_ERR_UNINITIALIZED =>                     c"Not initialized yet.",
        FSBANK_ERR_WRITER_FORMAT =>                     c"Chosen encode format is not supported by this FSB version.",
        FSBANK_WARN_CANNOTLOOP =>                       c"Source file is too short for seamless looping. Looping disabled.",
        FSBANK_WARN_IGNORED_FILTERHIGHFREQ =>           c"FSBANK_BUILD_FILTERHIGHFREQ flag ignored: feature only supported by XMA format.",
        FSBANK_WARN_IGNORED_DISABLESEEKING =>           c"FSBANK_BUILD_DISABLESEEKING flag ignored: feature only supported by XMA format.",
        FSBANK_WARN_FORCED_DONTWRITENAMES =>            c"FSBANK_BUILD_FSB5_DONTWRITENAMES flag forced: cannot write names when source is from memory.",
        FSBANK_ERR_ENCODER_FILE_NOTFOUND =>             c"External encoder dynamic library not found.",
        FSBANK_ERR_ENCODER_FILE_BAD =>                  c"External encoder dynamic library could not be loaded, possibly incorrect binary format, incorrect architecture, or file corruption.",
        FSBANK_WARN_IGNORED_ALIGN4K =>                  c"FSBANK_BUILD_ALIGN4K flag ignored: feature only supported by Opus, Vorbis, and FADPCM formats.",
        _ =>                                            c"Unknown error.",
    }.as_ptr()
}
