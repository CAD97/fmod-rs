#[cfg(doc)]
use fmod::*;
use {
    fmod::raw::*,
    std::{error::Error as _, ffi::CStr, fmt, num::NonZeroI32},
};

static_assertions::assert_type_eq_all!(i32, FMOD_RESULT);
static_assertions::const_assert_eq!(FMOD_OK, 0i32);

macro_rules! error_enum_struct {
    {$(
        $(#[$meta:meta])*
        $vis:vis enum $Name:ident: NonZeroI32 {$(
            $(#[$vmeta:meta])*
            $Variant:ident = $value:expr,
        )*}
    )*} => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        $vis struct $Name {
            raw: NonZeroI32,
        }

        impl $Name {
            raw! {
                pub const fn from_raw(raw: i32) -> Result<(), Self> {
                    match NonZeroI32::new(raw) {
                        Some(raw) => Err($Name { raw }),
                        None => Ok(()),
                    }
                }
            }
            raw! {
                pub const fn into_raw(self) -> i32 {
                    self.raw.get()
                }
            }

            // to clean up rustdoc, call this helper rather than inlining it
            const fn cook(raw: i32) -> Self {
                match Self::from_raw(raw) {
                    Err(this) => this,
                    Ok(()) => panic!("provided zero-valued FMOD_RESULT (FMOD_OK) as an error"),
                }
            }

            $(
                $(#[$vmeta])*
                #[allow(non_upper_case_globals)]
                pub const $Variant: Self = Self::cook($value);
            )*
        }

        impl fmt::Debug for $Name {
            #[deny(unreachable_patterns)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self {
                    $($Name::$Variant => f.debug_struct(stringify!($Variant)).finish(),)*
                    _ => f.debug_struct(stringify!($Name)).field("raw", &self.raw).finish(),
                }
            }
        }
    )*};
}

error_enum_struct! {
    /// An error that FMOD can emit.
    pub enum Error: NonZeroI32 {
        /// Tried to call a function on a data type that does not allow this type of functionality (ie calling [Sound::lock] on a streaming sound).
        BadCommand = FMOD_ERR_BADCOMMAND,
        /// Error trying to allocate a channel.
        ChannelAlloc = FMOD_ERR_CHANNEL_ALLOC,
        /// The specified channel has been reused to play another sound.
        ChannelStolen = FMOD_ERR_CHANNEL_STOLEN,
        /// DMA Failure. See debug output for more information.
        Dma = FMOD_ERR_DMA,
        /// DSP connection error. Connection possibly caused a cyclic dependency or connected dsps with incompatible buffer counts.
        DspConnection = FMOD_ERR_DSP_CONNECTION,
        /// DSP return code from a DSP process query callback. Tells mixer not to call the process callback and therefore not consume CPU. Use this to optimize the DSP graph.
        DspDontProcess = FMOD_ERR_DSP_DONTPROCESS,
        /// DSP Format error. A DSP unit may have attempted to connect to this network with the wrong format, or a matrix may have been set with the wrong size if the target unit has a specified channel map.
        DspFormat = FMOD_ERR_DSP_FORMAT,
        /// DSP is already in the mixer's DSP network. It must be removed before being reinserted or released.
        DspInUse = FMOD_ERR_DSP_INUSE,
        /// DSP connection error. Couldn't find the DSP unit specified.
        DspNotFound = FMOD_ERR_DSP_NOTFOUND,
        /// DSP operation error. Cannot perform operation on this DSP as it is reserved by the system.
        DspReserved = FMOD_ERR_DSP_RESERVED,
        /// DSP return code from a DSP process query callback. Tells mixer silence would be produced from read, so go idle and not consume CPU. Use this to optimize the DSP graph.
        DspSilence = FMOD_ERR_DSP_SILENCE,
        /// DSP operation cannot be performed on a DSP of this type.
        DspType = FMOD_ERR_DSP_TYPE,
        /// Error loading file.
        FileBad = FMOD_ERR_FILE_BAD,
        /// Couldn't perform seek operation. This is a limitation of the medium (ie netstreams) or the file format.
        FileCouldNotSeek = FMOD_ERR_FILE_COULDNOTSEEK,
        /// Media was ejected while reading.
        FileDiskEjected = FMOD_ERR_FILE_DISKEJECTED,
        /// End of file unexpectedly reached while trying to read essential data (truncated?).
        FileEof = FMOD_ERR_FILE_EOF,
        /// End of current chunk reached while trying to read data.
        FileEndOfData = FMOD_ERR_FILE_ENDOFDATA,
        /// File not found.
        FileNotFound = FMOD_ERR_FILE_NOTFOUND,
        /// Unsupported file or audio format.
        Format = FMOD_ERR_FORMAT,
        /// There is a version mismatch between the FMOD header and either the FMOD Studio library or the FMOD Low Level library.
        HeaderMismatch = FMOD_ERR_HEADER_MISMATCH,
        /// A HTTP error occurred. This is a catch-all for HTTP errors not listed elsewhere.
        Http = FMOD_ERR_HTTP,
        /// The specified resource requires authentication or is forbidden.
        HttpAccess = FMOD_ERR_HTTP_ACCESS,
        /// Proxy authentication is required to access the specified resource.
        HttpProxyAuth = FMOD_ERR_HTTP_PROXY_AUTH,
        /// A HTTP server error occurred.
        HttpServerError = FMOD_ERR_HTTP_SERVER_ERROR,
        /// The HTTP request timed out.
        HttpTimeout = FMOD_ERR_HTTP_TIMEOUT,
        /// FMOD was not initialized correctly to support this function.
        Initialization = FMOD_ERR_INITIALIZATION,
        /// Cannot call this command after System::init.
        Initialized = FMOD_ERR_INITIALIZED,
        /// An error occurred that wasn't supposed to. Contact support.
        Internal = FMOD_ERR_INTERNAL,
        /// Value passed in was a NaN, Inf or denormalized float.
        InvalidFloat = FMOD_ERR_INVALID_FLOAT,
        /// An invalid object handle was used.
        InvalidHandle = FMOD_ERR_INVALID_HANDLE,
        /// An invalid parameter was passed to this function.
        InvalidParam = FMOD_ERR_INVALID_PARAM,
        /// An invalid seek position was passed to this function.
        InvalidPosition = FMOD_ERR_INVALID_POSITION,
        /// An invalid speaker was passed to this function based on the current
        /// speaker mode.
        InvalidSpeaker = FMOD_ERR_INVALID_SPEAKER,
        /// The syncpoint did not come from this sound handle.
        InvalidSyncPoint = FMOD_ERR_INVALID_SYNCPOINT,
        /// Tried to call a function on a thread that is not supported.
        InvalidThread = FMOD_ERR_INVALID_THREAD,
        /// The vectors passed in are not unit length, or perpendicular.
        InvalidVector = FMOD_ERR_INVALID_VECTOR,
        /// Reached maximum audible playback count for this sound's soundgroup.
        MaxAudible = FMOD_ERR_MAXAUDIBLE,
        /// Not enough memory or resources.
        Memory = FMOD_ERR_MEMORY,
        /// Can't use [Mode::OpenMemoryPoint] on non PCM source data, or non mp3/xma/adpcm data if [Mode::CreateCompressedSample] was used.
        MemoryCantPoint = FMOD_ERR_MEMORY_CANTPOINT,
        /// Tried to call a command on a 2d sound when the command was meant for 3d sound.
        Needs3d = FMOD_ERR_NEEDS3D,
        /// Tried to use a feature that requires hardware support.
        NeedsHardware = FMOD_ERR_NEEDSHARDWARE,
        /// Couldn't connect to the specified host.
        NetConnect = FMOD_ERR_NET_CONNECT,
        /// A socket error occurred.  This is a catch-all for socket-related errors not listed elsewhere.
        NetSocketError = FMOD_ERR_NET_SOCKET_ERROR,
        /// The specified URL couldn't be resolved.
        NetUrl = FMOD_ERR_NET_URL,
        /// Operation on a non-blocking socket could not complete immediately.
        NetWouldBlock = FMOD_ERR_NET_WOULD_BLOCK,
        /// Operation could not be performed because specified sound/DSP connection is not ready.
        NotReady = FMOD_ERR_NOTREADY,
        /// Error initializing output device, but more specifically, the output device is already in use and cannot be reused.
        OutputAllocated = FMOD_ERR_OUTPUT_ALLOCATED,
        /// Error creating hardware sound buffer.
        OutputCreateBuffer = FMOD_ERR_OUTPUT_CREATEBUFFER,
        /// A call to a standard soundcard driver failed, which could possibly mean a bug in the driver or resources were missing or exhausted.
        OutputDriverCall = FMOD_ERR_OUTPUT_DRIVERCALL,
        /// Soundcard does not support the specified format.
        OutputFormat = FMOD_ERR_OUTPUT_FORMAT,
        /// Error initializing output device.
        OutputInit = FMOD_ERR_OUTPUT_INIT,
        /// The output device has no drivers installed. If pre-init, [Output::NoSound] is selected as the output mode. If post-init, the function just fails.
        OutputNoDrivers = FMOD_ERR_OUTPUT_NODRIVERS,
        /// An unspecified error has been returned from a plugin.
        Plugin = FMOD_ERR_PLUGIN,
        /// A requested output, dsp unit type or codec was not available.
        PluginMissing = FMOD_ERR_PLUGIN_MISSING,
        /// A resource that the plugin requires cannot be found. (ie the DLS file for MIDI playback)
        PluginResource = FMOD_ERR_PLUGIN_RESOURCE,
        /// A plugin was built with an unsupported SDK version.
        PluginVersion = FMOD_ERR_PLUGIN_VERSION,
        /// An error occurred trying to initialize the recording device.
        Record = FMOD_ERR_RECORD,
        /// Reverb properties cannot be set on this channel because a parent channelgroup owns the reverb connection.
        ReverbChannelGroup = FMOD_ERR_REVERB_CHANNELGROUP,
        /// Specified instance in [ReverbProperties] couldn't be set. Most likely because it is an invalid instance number or the reverb doesn't exist.
        ReverbInstance = FMOD_ERR_REVERB_INSTANCE,
        /// The error occurred because the sound referenced contains subsounds when it shouldn't have, or it doesn't contain subsounds when it should have. The operation may also not be able to be performed on a parent sound.
        SubSounds = FMOD_ERR_SUBSOUNDS,
        /// This subsound is already being used by another sound, you cannot have more than one parent to a sound. Null out the other parent's entry first.
        SubSoundAllocated = FMOD_ERR_SUBSOUND_ALLOCATED,
        /// Shared subsounds cannot be replaced or moved from their parent stream, such as when the parent stream is an FSB file.
        SubSoundCantMove = FMOD_ERR_SUBSOUND_CANTMOVE,
        /// The specified tag could not be found or there are no tags.
        TagNotFound = FMOD_ERR_TAGNOTFOUND,
        /// The sound created exceeds the allowable input channel count. This can be increased using the max_input_channels parameter in [System::set_software_format].
        TooManyChannels = FMOD_ERR_TOOMANYCHANNELS,
        /// The retrieved string is too long to fit in the supplied buffer and has been truncated.
        Truncated = FMOD_ERR_TRUNCATED,
        /// Something in FMOD hasn't been implemented when it should be! contact support!
        Unimplemented = FMOD_ERR_UNIMPLEMENTED,
        /// This command failed because [System::init] or [System::set_driver] was not called.
        Uninitialized = FMOD_ERR_UNINITIALIZED,
        /// A command issued was not supported by this object. Possibly a plugin without certain callbacks specified.
        Unsupported = FMOD_ERR_UNSUPPORTED,
        /// The version number of this file format is not supported.
        Version = FMOD_ERR_VERSION,
        /// The specified bank has already been loaded.
        EventAlreadyLoaded = FMOD_ERR_EVENT_ALREADY_LOADED,
        /// The live update connection failed due to the game already being connected.
        EventLiveUpdateBusy = FMOD_ERR_EVENT_LIVEUPDATE_BUSY,
        /// The live update connection failed due to the game data being out of sync with the tool.
        EventLiveUpdateMismatch = FMOD_ERR_EVENT_LIVEUPDATE_MISMATCH,
        /// The live update connection timed out.
        EventLiveUpdateTimeout = FMOD_ERR_EVENT_LIVEUPDATE_TIMEOUT,
        /// The requested event, parameter, bus or vca could not be found.
        EventNotFound = FMOD_ERR_EVENT_NOTFOUND,
        /// The [studio::System] object is not yet initialized.
        StudioUninitialized = FMOD_ERR_STUDIO_UNINITIALIZED,
        /// The specified resource is not loaded, so it can't be unloaded.
        StudioNotLoaded = FMOD_ERR_STUDIO_NOT_LOADED,
        /// An invalid string was passed to this function.
        InvalidString = FMOD_ERR_INVALID_STRING,
        /// The specified resource is already locked.
        AlreadyLocked = FMOD_ERR_ALREADY_LOCKED,
        /// The specified resource is not locked, so it can't be unlocked.
        NotLocked = FMOD_ERR_NOT_LOCKED,
        /// The specified recording driver has been disconnected.
        RecordDisconnected = FMOD_ERR_RECORD_DISCONNECTED,
        /// The length provided exceeds the allowable limit.
        TooManySamples = FMOD_ERR_TOOMANYSAMPLES,

        /// An error occurred in FMOD.rs that wasn't supposed to. Check the logs and open an issue.
        InternalRs = -1,
    }
}

/// Type alias for FMOD function results.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

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
