use {
    crate::raw::*,
    std::{error::Error as _, ffi::CStr, fmt, num::NonZeroI32},
};

static_assertions::assert_type_eq_all!(i32, FMOD_RESULT);
static_assertions::const_assert_eq!(FMOD_OK, 0i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    raw: NonZeroI32,
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

macro_rules! cook {
    ($raw:expr) => {
        match Self::from_raw($raw) {
            Some(this) => this,
            None => panic!("provided zero-valued FMOD_RESULT (FMOD_OK) as an error"),
        }
    };
}

#[allow(non_upper_case_globals)]
impl Error {
    raw! {
        pub const fn from_raw(raw: FMOD_RESULT) -> Option<Error> {
            match NonZeroI32::new(raw) {
                Some(raw) => Some(Error { raw }),
                None => None,
            }
        }
    }

    raw! {
        pub const fn into_raw(self) -> FMOD_RESULT {
            self.raw.get()
        }
    }

    /// Tried to call a function on a data type that does not allow this type
    /// of functionality (ie calling `Sound::lock` on a streaming sound).
    pub const BadCommand: Self = cook!(FMOD_ERR_BADCOMMAND);
    /// Error trying to allocate a channel.
    pub const ChannelAlloc: Self = cook!(FMOD_ERR_CHANNEL_ALLOC);
    /// The specified channel has been reused to play another sound.
    pub const ChannelStolen: Self = cook!(FMOD_ERR_CHANNEL_STOLEN);
    /// DMA Failure. See debug output for more information.
    pub const Dma: Self = cook!(FMOD_ERR_DMA);
    /// DSP connection error. Connection possibly caused a cyclic dependency or
    /// connected dsps with incompatible buffer counts.
    pub const DspConnection: Self = cook!(FMOD_ERR_DSP_CONNECTION);
    /// DSP return code from a DSP process query callback. Tells mixer not to
    /// call the process callback and therefore not consume CPU. Use this to
    /// optimize the DSP graph.
    pub const DspDontProcess: Self = cook!(FMOD_ERR_DSP_DONTPROCESS);
    /// DSP Format error. A DSP unit may have attempted to connect to this
    /// network with the wrong format, or a matrix may have been set with the
    /// wrong size if the target unit has a specified channel map.
    pub const DspFormat: Self = cook!(FMOD_ERR_DSP_FORMAT);
    /// DSP is already in the mixer's DSP network. It must be removed before
    /// being reinserted or released.
    pub const DspInUse: Self = cook!(FMOD_ERR_DSP_INUSE);
    /// DSP connection error. Couldn't find the DSP unit specified.
    pub const DspNotFound: Self = cook!(FMOD_ERR_DSP_NOTFOUND);
    /// DSP operation error. Cannot perform operation on this DSP as it is
    /// reserved by the system.
    pub const DspReserved: Self = cook!(FMOD_ERR_DSP_RESERVED);
    /// DSP return code from a DSP process query callback. Tells mixer silence
    /// would be produced from read, so go idle and not consume CPU. Use this to
    /// optimize the DSP graph.
    pub const DspSilence: Self = cook!(FMOD_ERR_DSP_SILENCE);
    /// DSP operation cannot be performed on a DSP of this type.
    pub const DspType: Self = cook!(FMOD_ERR_DSP_TYPE);
    /// Error loading file.
    pub const FileBad: Self = cook!(FMOD_ERR_FILE_BAD);
    /// Couldn't perform seek operation.
    /// This is a limitation of the medium (ie netstreams) or the file format.
    pub const FileCouldNotSeek: Self = cook!(FMOD_ERR_FILE_COULDNOTSEEK);
    /// Media was ejected while reading.
    pub const FileDiskEjected: Self = cook!(FMOD_ERR_FILE_DISKEJECTED);
    /// End of file unexpectedly reached while trying to read essential data
    /// (truncated?).
    pub const FileEof: Self = cook!(FMOD_ERR_FILE_EOF);
    /// End of current chunk reached while trying to read data.
    pub const FileEndOfData: Self = cook!(FMOD_ERR_FILE_ENDOFDATA);
    /// File not found.
    pub const FileNotFound: Self = cook!(FMOD_ERR_FILE_NOTFOUND);
    /// Unsupported file or audio format.
    pub const Format: Self = cook!(FMOD_ERR_FORMAT);
    /// There is a version mismatch between the FMOD header and either the
    /// FMOD Studio library or the FMOD Low Level library.
    pub const HeaderMismatch: Self = cook!(FMOD_ERR_HEADER_MISMATCH);
    /// A HTTP error occurred. This is a catch-all for HTTP errors not listed
    /// elsewhere.
    pub const Http: Self = cook!(FMOD_ERR_HTTP);
    /// The specified resource requires authentication or is forbidden.
    pub const HttpAccess: Self = cook!(FMOD_ERR_HTTP_ACCESS);
    /// Proxy authentication is required to access the specified resource.
    pub const HttpProxyAuth: Self = cook!(FMOD_ERR_HTTP_PROXY_AUTH);
    /// A HTTP server error occurred.
    pub const HttpServerError: Self = cook!(FMOD_ERR_HTTP_SERVER_ERROR);
    /// The HTTP request timed out.
    pub const HttpTimeout: Self = cook!(FMOD_ERR_HTTP_TIMEOUT);
    /// FMOD was not initialized correctly to support this function.
    pub const Initialization: Self = cook!(FMOD_ERR_INITIALIZATION);
    /// Cannot call this command after System::init.
    pub const Initialized: Self = cook!(FMOD_ERR_INITIALIZED);
    /// An error occurred that wasn't supposed to. Contact support.
    pub const Internal: Self = cook!(FMOD_ERR_INTERNAL);
    /// Value passed in was a NaN, Inf or denormalized float.
    pub const InvalidFloat: Self = cook!(FMOD_ERR_INVALID_FLOAT);
    /// An invalid object handle was used.
    pub const InvalidHandle: Self = cook!(FMOD_ERR_INVALID_HANDLE);
    /// An invalid parameter was passed to this function.
    pub const InvalidParam: Self = cook!(FMOD_ERR_INVALID_PARAM);
    /// An invalid seek position was passed to this function.
    pub const InvalidPosition: Self = cook!(FMOD_ERR_INVALID_POSITION);
    /// An invalid speaker was passed to this function based on the current
    /// speaker mode.
    pub const InvalidSpeaker: Self = cook!(FMOD_ERR_INVALID_SPEAKER);
    /// The syncpoint did not come from this sound handle.
    pub const InvalidSyncPoint: Self = cook!(FMOD_ERR_INVALID_SYNCPOINT);
    /// Tried to call a function on a thread that is not supported.
    pub const InvalidThread: Self = cook!(FMOD_ERR_INVALID_THREAD);
    /// The vectors passed in are not unit length, or perpendicular.
    pub const InvalidVector: Self = cook!(FMOD_ERR_INVALID_VECTOR);
    /// Reached maximum audible playback count for this sound's soundgroup.
    pub const MaxAudible: Self = cook!(FMOD_ERR_MAXAUDIBLE);
    /// Not enough memory or resources.
    pub const Memory: Self = cook!(FMOD_ERR_MEMORY);
    /// Can't use `FMOD_OPENMEMORY_POINT` on non PCM source data, or non
    /// mp3/xma/adpcm data if `FMOD_CREATECOMPRESSEDSAMPLE` was used.
    pub const MemoryCantPoint: Self = cook!(FMOD_ERR_MEMORY_CANTPOINT);
    /// Tried to call a command on a 2d sound when the command was meant for 3d
    /// sound.
    pub const Needs3d: Self = cook!(FMOD_ERR_NEEDS3D);
    /// Tried to use a feature that requires hardware support.
    pub const NeedsHardware: Self = cook!(FMOD_ERR_NEEDSHARDWARE);
    /// Couldn't connect to the specified host.
    pub const NetConnect: Self = cook!(FMOD_ERR_NET_CONNECT);
    /// A socket error occurred.  This is a catch-all for socket-related errors
    /// not listed elsewhere.
    pub const NetSocketError: Self = cook!(FMOD_ERR_NET_SOCKET_ERROR);
    /// The specified URL couldn't be resolved.
    pub const NetUrl: Self = cook!(FMOD_ERR_NET_URL);
    /// Operation on a non-blocking socket could not complete immediately.
    pub const NetWouldBlock: Self = cook!(FMOD_ERR_NET_WOULD_BLOCK);
    /// Operation could not be performed because specified sound/DSP connection
    /// is not ready.
    pub const NotReady: Self = cook!(FMOD_ERR_NOTREADY);
    /// Error initializing output device, but more specifically, the output
    /// device is already in use and cannot be reused.
    pub const OutputAllocated: Self = cook!(FMOD_ERR_OUTPUT_ALLOCATED);
    /// Error creating hardware sound buffer.
    pub const OutputCreateBuffer: Self = cook!(FMOD_ERR_OUTPUT_CREATEBUFFER);
    /// A call to a standard soundcard driver failed, which could possibly mean
    /// a bug in the driver or resources were missing or exhausted.
    pub const OutputDriverCall: Self = cook!(FMOD_ERR_OUTPUT_DRIVERCALL);
    /// Soundcard does not support the specified format.
    pub const OutputFormat: Self = cook!(FMOD_ERR_OUTPUT_FORMAT);
    /// Error initializing output device.
    pub const OutputInit: Self = cook!(FMOD_ERR_OUTPUT_INIT);
    /// The output device has no drivers installed. If pre-init,
    /// `FMOD_OUTPUT_NOSOUND` is selected as the output mode. If post-init, the
    /// function just fails.
    pub const OutputNoDrivers: Self = cook!(FMOD_ERR_OUTPUT_NODRIVERS);
    /// An unspecified error has been returned from a plugin.
    pub const Plugin: Self = cook!(FMOD_ERR_PLUGIN);
    /// A requested output, dsp unit type or codec was not available.
    pub const PluginMissing: Self = cook!(FMOD_ERR_PLUGIN_MISSING);
    /// A resource that the plugin requires cannot be found. (ie the DLS file
    /// for MIDI playback)
    pub const PluginResource: Self = cook!(FMOD_ERR_PLUGIN_RESOURCE);
    /// A plugin was built with an unsupported SDK version.
    pub const PluginVersion: Self = cook!(FMOD_ERR_PLUGIN_VERSION);
    /// An error occurred trying to initialize the recording device.
    pub const Record: Self = cook!(FMOD_ERR_RECORD);
    /// Reverb properties cannot be set on this channel because a parent
    /// channelgroup owns the reverb connection.
    pub const ReverbChannelGroup: Self = cook!(FMOD_ERR_REVERB_CHANNELGROUP);
    /// Specified instance in FMOD_REVERB_PROPERTIES couldn't be set. Most
    /// likely because it is an invalid instance number or the reverb doesn't
    /// exist.
    pub const ReverbInstance: Self = cook!(FMOD_ERR_REVERB_INSTANCE);
    /// The error occurred because the sound referenced contains subsounds when
    /// it shouldn't have, or it doesn't contain subsounds when it should have.
    /// The operation may also not be able to be performed on a parent sound.
    pub const SubSounds: Self = cook!(FMOD_ERR_SUBSOUNDS);
    /// This subsound is already being used by another sound, you cannot have
    /// more than one parent to a sound. Null out the other parent's entry
    /// first.
    pub const SubSoundAllocated: Self = cook!(FMOD_ERR_SUBSOUND_ALLOCATED);
    /// Shared subsounds cannot be replaced or moved from their parent stream,
    /// such as when the parent stream is an FSB file.
    pub const SubSoundCantMove: Self = cook!(FMOD_ERR_SUBSOUND_CANTMOVE);
    /// The specified tag could not be found or there are no tags.
    pub const TagNotFound: Self = cook!(FMOD_ERR_TAGNOTFOUND);
    /// The sound created exceeds the allowable input channel count. This can be
    /// increased using the `maxinputchannels` parameter in
    /// `System::setSoftwareFormat`.
    pub const TooManyChannels: Self = cook!(FMOD_ERR_TOOMANYCHANNELS);
    /// The retrieved string is too long to fit in the supplied buffer and has
    /// been truncated.
    pub const Truncated: Self = cook!(FMOD_ERR_TRUNCATED);
    /// Something in FMOD hasn't been implemented when it should be! contact
    /// support!
    pub const Unimplemented: Self = cook!(FMOD_ERR_UNIMPLEMENTED);
    /// This command failed because `System::init` or `System::setDriver` was
    /// not called.
    pub const Uninitialized: Self = cook!(FMOD_ERR_UNINITIALIZED);
    /// A command issued was not supported by this object. Possibly a plugin
    /// without certain callbacks specified.
    pub const Unsupported: Self = cook!(FMOD_ERR_UNSUPPORTED);
    /// The version number of this file format is not supported.
    pub const Version: Self = cook!(FMOD_ERR_VERSION);
    /// The specified bank has already been loaded.
    pub const EventAlreadyLoaded: Self = cook!(FMOD_ERR_EVENT_ALREADY_LOADED);
    /// The live update connection failed due to the game already being
    /// connected.
    pub const EventLiveUpdateBusy: Self = cook!(FMOD_ERR_EVENT_LIVEUPDATE_BUSY);
    /// The live update connection failed due to the game data being out of sync
    /// with the tool.
    pub const EventLiveUpdateMismatch: Self = cook!(FMOD_ERR_EVENT_LIVEUPDATE_MISMATCH);
    /// The live update connection timed out.
    pub const EventLiveUpdateTimeout: Self = cook!(FMOD_ERR_EVENT_LIVEUPDATE_TIMEOUT);
    /// The requested event, parameter, bus or vca could not be found.
    pub const EventNotFound: Self = cook!(FMOD_ERR_EVENT_NOTFOUND);
    /// The Studio::System object is not yet initialized.
    pub const StudioUninitialized: Self = cook!(FMOD_ERR_STUDIO_UNINITIALIZED);
    /// The specified resource is not loaded, so it can't be unloaded.
    pub const StudioNotLoaded: Self = cook!(FMOD_ERR_STUDIO_NOT_LOADED);
    /// An invalid string was passed to this function.
    pub const InvalidString: Self = cook!(FMOD_ERR_INVALID_STRING);
    /// The specified resource is already locked.
    pub const AlreadyLocked: Self = cook!(FMOD_ERR_ALREADY_LOCKED);
    /// The specified resource is not locked, so it can't be unlocked.
    pub const NotLocked: Self = cook!(FMOD_ERR_NOT_LOCKED);
    /// The specified recording driver has been disconnected.
    pub const RecordDisconnected: Self = cook!(FMOD_ERR_RECORD_DISCONNECTED);
    /// The length provided exceeds the allowable limit.
    pub const TooManySamples: Self = cook!(FMOD_ERR_TOOMANYSAMPLES);

    /// An error occurred in FMOD.rs that wasn't supposed to. Check the logs and
    /// open an issue.
    pub const InternalRs: Self = cook!(-1);
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        if *self == Error::InternalRs {
            return "An error occurred in FMOD.rs that wasn't supposed to. Check the logs and open an issue.";
        }

        // SAFETY: FMOD_ErrorString is a C `static` function which thus isn't
        // bindgen'd, but hand implemented in fmod-core-sys. As such, we're
        // 100% sure that it always returns valid nul-terminated ASCII.
        unsafe {
            let error_string = CStr::from_ptr(FMOD_ErrorString(self.raw.into()));
            error_string.to_str().unwrap_unchecked()
        }
    }
}

impl fmt::Display for Error {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())
    }
}

#[cfg(feature = "fmod_debug_is_tracing")]
pub(crate) fn fmod_debug_install_tracing() -> Result<()> {
    use {
        crate::{DebugFlags, DebugMode},
        std::ptr,
        tracing::Level,
    };

    // From my experience, FMOD_DEBUG_TYPE_MEMORY, _FILE, _CODEC are *very very*
    // verbose, and are basically TRACE level detail. Also very interestingly,
    // FMOD_DEBUG_TYPE_TRACE has yet to make a peep in any implemented example.
    // As such, we map _TRACE to tracing Level::DEBUG and _MEMORY, _FILE, _CODEC
    // to tracing Level::TRACE. This looks backwards on first glance, but seems
    // to match their use in practice.

    fn on_fmod_debug(
        flags: DebugFlags,
        file: Option<&str>,
        line: i32,
        func: Option<&str>,
        message: Option<&str>,
    ) {
        if flags.is_set(DebugFlags::TypeMemory) {
            tracing::trace!(target: "fmod::memory", parent: &crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::TypeFile) {
            tracing::trace!(target: "fmod::file", parent: &crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::TypeCodec) {
            tracing::trace!(target: "fmod::codec", parent: &crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::TypeTrace) {
            tracing::debug!(target: "fmod", parent: &crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::LevelLog) {
            tracing::info!(target: "fmod", parent: &crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::LevelWarning) {
            tracing::warn!(target: "fmod", parent: &crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::LevelError) {
            tracing::error!(target: "fmod", parent: &crate::span(), file, line, func, message)
        } else {
            panic!("FMOD debug callback called without message level")
        };
    }

    let mut debug_flags = DebugFlags::LevelNone;

    // Enable logging for each level if and only if they're `enabled!`. This
    // allows us to tell FMOD what we're actually interested in hearing about.
    // The FMOD default filter is `fmod=INFO` (DebugFlags::LevelLog). Enabling
    // fmod::memory/file/codec=trace should only be done when debugging specific
    // issues that require tracing that module's execution.

    // This techically relies on the default subscriber in this context always
    // being the subscriber that we log to for correctness, but the advantage of
    // not enabling logs FMOD-side at all outweigh the inability to dynamically
    // switch subscriber / filter in this case.

    if tracing::enabled!(target: "fmod", Level::ERROR, { file, line, func, message }) {
        debug_flags = DebugFlags::LevelError;
    }
    if tracing::enabled!(target: "fmod", Level::WARN, { file, line, func, message }) {
        debug_flags = DebugFlags::LevelWarning;
    }
    if tracing::enabled!(target: "fmod", Level::INFO, { file, line, func, message }) {
        debug_flags = DebugFlags::LevelLog;
    }
    if tracing::enabled!(target: "fmod", Level::DEBUG, { file, line, func, message }) {
        debug_flags |= DebugFlags::TypeTrace;
    }
    if tracing::enabled!(target: "fmod::memory", Level::TRACE, { file, line, func, message }) {
        debug_flags |= DebugFlags::TypeMemory;
    }
    if tracing::enabled!(target: "fmod::file", Level::TRACE, { file, line, func, message }) {
        debug_flags |= DebugFlags::TypeFile;
    }
    if tracing::enabled!(target: "fmod::codec", Level::TRACE, { file, line, func, message }) {
        debug_flags |= DebugFlags::TypeCodec;
    }

    let flags = DebugFlags::into_raw(debug_flags);
    let mode = DebugMode::into_raw(DebugMode::Callback);
    unsafe extern "C" fn callback(
        flags: FMOD_DEBUG_FLAGS,
        file: *const std::os::raw::c_char,
        line: std::os::raw::c_int,
        func: *const std::os::raw::c_char,
        message: *const std::os::raw::c_char,
    ) -> FMOD_RESULT {
        match std::panic::catch_unwind(|| {
            // NB: FMOD claims to only deal in UTF-8, so this _should_
            //     not make an owned String; to_string_lossy is only
            //     there as a precaution, and logging is turned off
            //     completely by FMOD in release builds, so this won't
            //     impact release performance anyway. Peace of mind 😊
            let flags = DebugFlags::from_raw(flags);
            let file = ptr::NonNull::new(file as *mut _)
                .map(|file| CStr::from_ptr(file.as_ptr()).to_string_lossy());
            let func = ptr::NonNull::new(func as *mut _)
                .map(|func| CStr::from_ptr(func.as_ptr()).to_string_lossy());
            let message = ptr::NonNull::new(message as *mut _)
                .map(|message| CStr::from_ptr(message.as_ptr()).to_string_lossy());
            on_fmod_debug(
                flags,
                file.as_deref().map(str::trim_end),
                line,
                func.as_deref().map(str::trim_end),
                message.as_deref().map(str::trim_end),
            )
        }) {
            Ok(()) => FMOD_OK,
            Err(e) => {
                if let Some(e) = cool_asserts::get_panic_message(&e) {
                    tracing::error!(
                        parent: &crate::span(),
                        "FMOD.rs panicked in a callback: {e}"
                    );
                } else {
                    tracing::error!(parent: &crate::span(), "FMOD.rs panicked in a callback");
                }
                Error::into_raw(Error::InternalRs)
            },
        }
    }

    let filename = ptr::null();

    let result = unsafe { FMOD_Debug_Initialize(flags, mode, Some(callback), filename) };
    if let Some(error) = Error::from_raw(result) {
        match error {
            Error::Unsupported => {
                tracing::info!(parent: &crate::span(), "FMOD logging disabled");
                Ok(())
            },
            error => Err(error),
        }
    } else {
        Ok(())
    }
}
