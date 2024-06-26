#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

include!("../inc/bindings.rs");

// fmod_errors.h `static const char *FMOD_ErrorString(FMOD_RESULT errcode)`
#[inline]
#[allow(clippy::missing_safety_doc)]
pub const unsafe fn FMOD_ErrorString(errcode: FMOD_RESULT) -> *const std::ffi::c_char {
    match errcode {
        FMOD_OK =>                            c"No errors.",
        FMOD_ERR_BADCOMMAND =>                c"Tried to call a function on a data type that does not allow this type of functionality (ie calling Sound::lock on a streaming sound).",
        FMOD_ERR_CHANNEL_ALLOC =>             c"Error trying to allocate a channel.",
        FMOD_ERR_CHANNEL_STOLEN =>            c"The specified channel has been reused to play another sound.",
        FMOD_ERR_DMA =>                       c"DMA Failure.  See debug output for more information.",
        FMOD_ERR_DSP_CONNECTION =>            c"DSP connection error.  Connection possibly caused a cyclic dependency or connected dsps with incompatible buffer counts.",
        FMOD_ERR_DSP_DONTPROCESS =>           c"DSP return code from a DSP process query callback.  Tells mixer not to call the process callback and therefore not consume CPU.  Use this to optimize the DSP graph.",
        FMOD_ERR_DSP_FORMAT =>                c"DSP Format error.  A DSP unit may have attempted to connect to this network with the wrong format, or a matrix may have been set with the wrong size if the target unit has a specified channel map.",
        FMOD_ERR_DSP_INUSE =>                 c"DSP is already in the mixer's DSP network. It must be removed before being reinserted or released.",
        FMOD_ERR_DSP_NOTFOUND =>              c"DSP connection error.  Couldn't find the DSP unit specified.",
        FMOD_ERR_DSP_RESERVED =>              c"DSP operation error.  Cannot perform operation on this DSP as it is reserved by the system.",
        FMOD_ERR_DSP_SILENCE =>               c"DSP return code from a DSP process query callback.  Tells mixer silence would be produced from read, so go idle and not consume CPU.  Use this to optimize the DSP graph.",
        FMOD_ERR_DSP_TYPE =>                  c"DSP operation cannot be performed on a DSP of this type.",
        FMOD_ERR_FILE_BAD =>                  c"Error loading file.",
        FMOD_ERR_FILE_COULDNOTSEEK =>         c"Couldn't perform seek operation.  This is a limitation of the medium (ie netstreams) or the file format.",
        FMOD_ERR_FILE_DISKEJECTED =>          c"Media was ejected while reading.",
        FMOD_ERR_FILE_EOF =>                  c"End of file unexpectedly reached while trying to read essential data (truncated?).",
        FMOD_ERR_FILE_ENDOFDATA =>            c"End of current chunk reached while trying to read data.",
        FMOD_ERR_FILE_NOTFOUND =>             c"File not found.",
        FMOD_ERR_FORMAT =>                    c"Unsupported file or audio format.",
        FMOD_ERR_HEADER_MISMATCH =>           c"There is a version mismatch between the FMOD header and either the FMOD Studio library or the FMOD Low Level library.",
        FMOD_ERR_HTTP =>                      c"A HTTP error occurred. This is a catch-all for HTTP errors not listed elsewhere.",
        FMOD_ERR_HTTP_ACCESS =>               c"The specified resource requires authentication or is forbidden.",
        FMOD_ERR_HTTP_PROXY_AUTH =>           c"Proxy authentication is required to access the specified resource.",
        FMOD_ERR_HTTP_SERVER_ERROR =>         c"A HTTP server error occurred.",
        FMOD_ERR_HTTP_TIMEOUT =>              c"The HTTP request timed out.",
        FMOD_ERR_INITIALIZATION =>            c"FMOD was not initialized correctly to support this function.",
        FMOD_ERR_INITIALIZED =>               c"Cannot call this command after System::init.",
        FMOD_ERR_INTERNAL =>                  c"An error occurred in the FMOD system. Use the logging version of FMOD for more information.",
        FMOD_ERR_INVALID_FLOAT =>             c"Value passed in was a NaN, Inf or denormalized float.",
        FMOD_ERR_INVALID_HANDLE =>            c"An invalid object handle was used.",
        FMOD_ERR_INVALID_PARAM =>             c"An invalid parameter was passed to this function.",
        FMOD_ERR_INVALID_POSITION =>          c"An invalid seek position was passed to this function.",
        FMOD_ERR_INVALID_SPEAKER =>           c"An invalid speaker was passed to this function based on the current speaker mode.",
        FMOD_ERR_INVALID_SYNCPOINT =>         c"The syncpoint did not come from this sound handle.",
        FMOD_ERR_INVALID_THREAD =>            c"Tried to call a function on a thread that is not supported.",
        FMOD_ERR_INVALID_VECTOR =>            c"The vectors passed in are not unit length, or perpendicular.",
        FMOD_ERR_MAXAUDIBLE =>                c"Reached maximum audible playback count for this sound's soundgroup.",
        FMOD_ERR_MEMORY =>                    c"Not enough memory or resources.",
        FMOD_ERR_MEMORY_CANTPOINT =>          c"Can't use FMOD_OPENMEMORY_POINT on non PCM source data, or non mp3/xma/adpcm data if FMOD_CREATECOMPRESSEDSAMPLE was used.",
        FMOD_ERR_NEEDS3D =>                   c"Tried to call a command on a 2d sound when the command was meant for 3d sound.",
        FMOD_ERR_NEEDSHARDWARE =>             c"Tried to use a feature that requires hardware support.",
        FMOD_ERR_NET_CONNECT =>               c"Couldn't connect to the specified host.",
        FMOD_ERR_NET_SOCKET_ERROR =>          c"A socket error occurred.  This is a catch-all for socket-related errors not listed elsewhere.",
        FMOD_ERR_NET_URL =>                   c"The specified URL couldn't be resolved.",
        FMOD_ERR_NET_WOULD_BLOCK =>           c"Operation on a non-blocking socket could not complete immediately.",
        FMOD_ERR_NOTREADY =>                  c"Operation could not be performed because specified sound/DSP connection is not ready.",
        FMOD_ERR_OUTPUT_ALLOCATED =>          c"Error initializing output device, but more specifically, the output device is already in use and cannot be reused.",
        FMOD_ERR_OUTPUT_CREATEBUFFER =>       c"Error creating hardware sound buffer.",
        FMOD_ERR_OUTPUT_DRIVERCALL =>         c"A call to a standard soundcard driver failed, which could possibly mean a bug in the driver or resources were missing or exhausted.",
        FMOD_ERR_OUTPUT_FORMAT =>             c"Soundcard does not support the specified format.",
        FMOD_ERR_OUTPUT_INIT =>               c"Error initializing output device.",
        FMOD_ERR_OUTPUT_NODRIVERS =>          c"The output device has no drivers installed.  If pre-init, FMOD_OUTPUT_NOSOUND is selected as the output mode.  If post-init, the function just fails.",
        FMOD_ERR_PLUGIN =>                    c"An unspecified error has been returned from a plugin.",
        FMOD_ERR_PLUGIN_MISSING =>            c"A requested output, dsp unit type or codec was not available.",
        FMOD_ERR_PLUGIN_RESOURCE =>           c"A resource that the plugin requires cannot be found. (ie the DLS file for MIDI playback)",
        FMOD_ERR_PLUGIN_VERSION =>            c"A plugin was built with an unsupported SDK version.",
        FMOD_ERR_RECORD =>                    c"An error occurred trying to initialize the recording device.",
        FMOD_ERR_REVERB_CHANNELGROUP =>       c"Reverb properties cannot be set on this channel because a parent channelgroup owns the reverb connection.",
        FMOD_ERR_REVERB_INSTANCE =>           c"Specified instance in FMOD_REVERB_PROPERTIES couldn't be set. Most likely because it is an invalid instance number or the reverb doesn't exist.",
        FMOD_ERR_SUBSOUNDS =>                 c"The error occurred because the sound referenced contains subsounds when it shouldn't have, or it doesn't contain subsounds when it should have.  The operation may also not be able to be performed on a parent sound.",
        FMOD_ERR_SUBSOUND_ALLOCATED =>        c"This subsound is already being used by another sound, you cannot have more than one parent to a sound.  Null out the other parent's entry first.",
        FMOD_ERR_SUBSOUND_CANTMOVE =>         c"Shared subsounds cannot be replaced or moved from their parent stream, such as when the parent stream is an FSB file.",
        FMOD_ERR_TAGNOTFOUND =>               c"The specified tag could not be found or there are no tags.",
        FMOD_ERR_TOOMANYCHANNELS =>           c"The sound created exceeds the allowable input channel count.  This can be increased using the 'maxinputchannels' parameter in System::setSoftwareFormat.",
        FMOD_ERR_TRUNCATED =>                 c"The retrieved string is too long to fit in the supplied buffer and has been truncated.",
        FMOD_ERR_UNIMPLEMENTED =>             c"Something in FMOD hasn't been implemented when it should be. Contact support.",
        FMOD_ERR_UNINITIALIZED =>             c"This command failed because System::init or System::setDriver was not called.",
        FMOD_ERR_UNSUPPORTED =>               c"A command issued was not supported by this object.  Possibly a plugin without certain callbacks specified.",
        FMOD_ERR_VERSION =>                   c"The version number of this file format is not supported.",
        FMOD_ERR_EVENT_ALREADY_LOADED =>      c"The specified bank has already been loaded.",
        FMOD_ERR_EVENT_LIVEUPDATE_BUSY =>     c"The live update connection failed due to the game already being connected.",
        FMOD_ERR_EVENT_LIVEUPDATE_MISMATCH => c"The live update connection failed due to the game data being out of sync with the tool.",
        FMOD_ERR_EVENT_LIVEUPDATE_TIMEOUT =>  c"The live update connection timed out.",
        FMOD_ERR_EVENT_NOTFOUND =>            c"The requested event, parameter, bus or vca could not be found.",
        FMOD_ERR_STUDIO_UNINITIALIZED =>      c"The Studio::System object is not yet initialized.",
        FMOD_ERR_STUDIO_NOT_LOADED =>         c"The specified resource is not loaded, so it can't be unloaded.",
        FMOD_ERR_INVALID_STRING =>            c"An invalid string was passed to this function.",
        FMOD_ERR_ALREADY_LOCKED =>            c"The specified resource is already locked.",
        FMOD_ERR_NOT_LOCKED =>                c"The specified resource is not locked, so it can't be unlocked.",
        FMOD_ERR_RECORD_DISCONNECTED =>       c"The specified recording driver has been disconnected.",
        FMOD_ERR_TOOMANYSAMPLES =>            c"The length provided exceeds the allowable limit.",
        _ =>                                  c"Unknown error.",
    }.as_ptr()
}
