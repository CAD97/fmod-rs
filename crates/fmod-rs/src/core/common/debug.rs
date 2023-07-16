//! Functionality relating to FMOD's debug logging.

use {
    crate::utils::{catch_user_unwind, str_from_nonnull_unchecked},
    fmod::{raw::*, *},
    std::{
        ffi::{c_char, c_int},
        ptr,
        sync::Once,
    },
};

static DEBUG_LAYER_INITIALIZED: Once = Once::new();

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
#[cfg_attr(
    feature = "log",
    doc = r#"
<div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
<span class="emoji">🦀</span><span>
FMOD.rs automatically initializes FMOD's logs to go to the log crate. Manually
initializing FMOD debugging will override this behavior.
</span></div></div>
"#
)]
pub fn initialize(flags: DebugFlags) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();

    let mut result = Ok(());
    DEBUG_LAYER_INITIALIZED.call_once(|| {
        result = ffi!(FMOD_Debug_Initialize(
            flags.into_raw(),
            DebugMode::Tty.into_raw(),
            None,
            ptr::null()
        ));
    });

    result
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
#[cfg_attr(
    feature = "log",
    doc = r#"
<div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
<span class="emoji">🦀</span><span>
FMOD.rs automatically initializes FMOD's logs to go to the log crate. Manually
initializing FMOD debugging will override this behavior.
</span></div></div>
"#
)]
pub fn initialize_callback<D: DebugCallback>(flags: DebugFlags) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();

    let mut result = Ok(());
    DEBUG_LAYER_INITIALIZED.call_once(|| {
        result = ffi!(FMOD_Debug_Initialize(
            flags.into_raw(),
            DebugMode::Callback.into_raw(),
            Some(debug_callback::<D>),
            ptr::null()
        ));
    });

    result
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
/// - [DebugFlags::LevelLog] produces informational, warning and error messages.
/// - [DebugFlags::LevelWarning] produces warnings and error messages.
/// - [DebugFlags::LevelError] produces error messages only.
///
#[cfg_attr(
    feature = "log",
    doc = r#"
<div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
<span class="emoji">🦀</span><span>
FMOD.rs automatically initializes FMOD's logs to go to the log crate. Manually
initializing FMOD debugging will override this behavior.
</span></div></div>
"#
)]
pub fn initialize_file(flags: DebugFlags, file: &CStr8) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();

    let mut result = Ok(());
    DEBUG_LAYER_INITIALIZED.call_once(|| {
        result = ffi!(FMOD_Debug_Initialize(
            flags.into_raw(),
            DebugMode::File.into_raw(),
            None,
            file.as_ptr() as _,
        ));
    });

    result
}

#[cfg(feature = "log")]
/// Initialize the FMOD debug layer to write to the global logger via
/// [`DebugViaRust`].
///
/// If FMOD debugging is statically disabled, logs this via `info!`.
///
/// This is done automatically for you when an FMOD System is first created.
/// This is only required if changing log filtering or the FMOD debug layer's
/// state at runtime.
pub fn initialize_log() {
    match initialize_callback::<DebugViaRust>(DebugViaRust::ideal_debug_flags()) {
        Ok(()) => (),
        Err(error) => handle_init_failure(error),
    }
}

/// Initialize the FMOD debug layer only if it hasn't been already initialized
/// and skipping the global read lock against racing system init.
/// Called as part of system init.
pub(crate) unsafe fn initialize_default() {
    let mut result = Ok(());
    DEBUG_LAYER_INITIALIZED.call_once(|| {
        #[cfg(feature = "log")]
        {
            result = ffi!(FMOD_Debug_Initialize(
                DebugViaRust::ideal_debug_flags().into_raw(),
                DebugMode::Callback.into_raw(),
                Some(debug_callback::<DebugViaRust>),
                ptr::null(),
            ));
        }
    });

    match result {
        Ok(()) => (),
        Err(error) => handle_init_failure(error),
    }
}

// -------------------------------------------------------------------------------------------------

/// Callback for debug messages when using the logging version of FMOD.
///
/// This callback will fire directly from the log line, as such it can be
/// from any thread.
pub trait DebugCallback {
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
    ) -> Result;
}

unsafe extern "system" fn debug_callback<C: DebugCallback>(
    flags: FMOD_DEBUG_FLAGS,
    file: *const c_char,
    line: c_int,
    func: *const c_char,
    msg: *const c_char,
) -> FMOD_RESULT {
    let flags = DebugFlags::from_raw(flags);
    // SAFETY: these strings are usually produced directly by FMOD, so they
    // should *actually* be guaranteed to be UTF-8 like FMOD claims.
    // However, plugins can also call this function, so we can't be sure.
    // Plugins are developer controlled, so assuming UTF-8 is probably fine.
    let cstr = |s: *const c_char| {
        ptr::NonNull::new(s as *mut _)
            .map(|x| str_from_nonnull_unchecked(x))
            .map(str::trim_end)
    };
    let file = cstr(file);
    let func = cstr(func);
    let message = cstr(msg);
    catch_user_unwind(|| C::log(flags, file, line, func, message)).into_raw()
}

// -------------------------------------------------------------------------------------------------

raw! {
    enum_struct! {
        /// Specify the destination of log output when using the logging version of FMOD.
        ///
        /// TTY destination can vary depending on platform, common examples include the Visual Studio / Xcode output window, stderr and LogCat.
        pub enum DebugMode: FMOD_DEBUG_MODE {
            /// Default log location per platform, i.e. Visual Studio output window, stderr, LogCat, etc.
            Tty      = FMOD_DEBUG_MODE_TTY,
            /// Write log to specified file path.
            File     = FMOD_DEBUG_MODE_FILE,
            /// Call specified callback with log information.
            Callback = FMOD_DEBUG_MODE_CALLBACK,
        }
    }
}

flags! {
    /// Specify the requested information to be output when using the logging version of FMOD.
    pub struct DebugFlags: FMOD_DEBUG_FLAGS {
        /// Disable all messages.
        LevelNone          = FMOD_DEBUG_LEVEL_NONE,
        /// Enable only error messages.
        LevelError         = FMOD_DEBUG_LEVEL_ERROR,
        /// Enable warning and error messages.
        LevelWarning       = FMOD_DEBUG_LEVEL_WARNING,
        #[default]
        /// Enable informational, warning and error messages (default).
        LevelLog           = FMOD_DEBUG_LEVEL_LOG,
        /// Verbose logging for memory operations, only use this if you are debugging a memory related issue.
        TypeMemory         = FMOD_DEBUG_TYPE_MEMORY,
        /// Verbose logging for file access, only use this if you are debugging a file related issue.
        TypeFile           = FMOD_DEBUG_TYPE_FILE,
        /// Verbose logging for codec initialization, only use this if you are debugging a codec related issue.
        TypeCodec          = FMOD_DEBUG_TYPE_CODEC,
        /// Verbose logging for internal errors, use this for tracking the origin of error codes.
        TypeTrace          = FMOD_DEBUG_TYPE_TRACE,
        /// Display the time stamp of the log message in milliseconds.
        DisplayTimestamps  = FMOD_DEBUG_DISPLAY_TIMESTAMPS,
        /// Display the source code file and line number for where the message originated.
        DisplayLinenumbers = FMOD_DEBUG_DISPLAY_LINENUMBERS,
        /// Display the thread ID of the calling function that generated the message.
        DisplayThread      = FMOD_DEBUG_DISPLAY_THREAD,
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(feature = "log")]
/// A [`DebugCallback`] sink that hooks up FMOD debug messages into [log].
///
/// <table>
/// <tr> <th>FMOD Debug Flag</th> <th>Log Level</th> <th>Log Target</th>   </tr>
/// <tr> <td>TypeMemory</td>      <td>Trace</td>     <td>fmod::memory</td> </tr>
/// <tr> <td>TypeFile</td>        <td>Trace</td>     <td>fmod::file</td>   </tr>
/// <tr> <td>TypeCodec</td>       <td>Trace</td>     <td>fmod::codec</td>  </tr>
/// <tr> <td>TypeTrace</td>       <td>Trace</td>     <td>fmod::trace</td>  </tr>
/// <tr> <td>LevelLog</td>        <td>Info</td>      <td>fmod</td>         </tr>
/// <tr> <td>LevelWarning</td>    <td>Warn</td>      <td>fmod</td>         </tr>
/// <tr> <td>LevelError</td>      <td>Error</td>     <td>fmod</td>         </tr>
/// </table>
///
/// The `Display*` flags are ignored and left to the decision of the logger.
///
/// Additionally, FMOD.rs traces handle creation/release in target fmod::handle.
pub enum DebugViaRust {}

#[cfg(feature = "log")]
impl DebugCallback for DebugViaRust {
    fn log(
        flags: DebugFlags,
        file: Option<&str>,
        line: i32,
        func: Option<&str>,
        message: Option<&str>,
    ) -> Result {
        let mut log = log::Record::builder();
        let message = message.unwrap_or_default();
        log.file(file).line(Some(line as u32)).module_path(func);

        // extract level from flags
        if flags.is_set(DebugFlags::LevelLog) {
            log.level(log::Level::Info);
        } else if flags.is_set(DebugFlags::LevelWarning) {
            log.level(log::Level::Warn);
        } else if flags.is_set(DebugFlags::LevelError) {
            log.level(log::Level::Error);
        } else {
            // no explicit level set, so leave the default (Log/Info)
        }

        // extract target from flags
        if flags.is_set(DebugFlags::TypeMemory) {
            log.level(log::Level::Trace).target("fmod::memory");
        } else if flags.is_set(DebugFlags::TypeFile) {
            log.level(log::Level::Trace).target("fmod::file");
        } else if flags.is_set(DebugFlags::TypeCodec) {
            log.level(log::Level::Trace).target("fmod::codec");
        } else if flags.is_set(DebugFlags::TypeTrace) {
            log.level(log::Level::Trace).target("fmod::trace");
        } else {
            // no explicit type set, so we use a generic fmod target
            log.target("fmod");
        };

        log::logger().log(&log.args(format_args!("{message}")).build());
        Ok(())
    }
}

#[cfg(feature = "log")]
impl DebugViaRust {
    /// Create [DebugFlags] enabling only debug logging which is enabled by
    /// the current logger.
    ///
    /// FMOD's default filter is [DebugFlags::LevelLog], which is equivalent
    /// to an env filter of `fmod=INFO` when using [`DebugViaRust`].
    /// Enabling the `fmod::memory`/`fmod::file`/`fmod::codec` targets should
    /// only be done when debugging specific issues that require tracing that
    /// area's execution; these are truly verbose trace level logging targets.
    pub fn ideal_debug_flags() -> DebugFlags {
        use log::{log_enabled, Level};
        let mut debug_flags = DebugFlags::LevelNone;

        if log_enabled!(target: "fmod", Level::Error) {
            debug_flags = DebugFlags::LevelError;
        }
        if log_enabled!(target: "fmod", Level::Warn) {
            debug_flags = DebugFlags::LevelWarning;
        }
        if log_enabled!(target: "fmod", Level::Info) {
            debug_flags = DebugFlags::LevelLog;
        }
        if log_enabled!(target: "fmod::trace", Level::Trace) {
            debug_flags |= DebugFlags::TypeTrace;
        }
        if log_enabled!(target: "fmod::memory", Level::Trace) {
            debug_flags |= DebugFlags::TypeMemory;
        }
        if log_enabled!(target: "fmod::file", Level::Trace) {
            debug_flags |= DebugFlags::TypeFile;
        }
        if log_enabled!(target: "fmod::codec", Level::Trace) {
            debug_flags |= DebugFlags::TypeCodec;
        }

        debug_flags
    }
}

#[cfg(feature = "log")]
fn handle_init_failure(error: Error) {
    match error {
        Error::Unsupported => log::info!("FMOD debug disabled"),
        error => whoops!("Error during FMOD debug initialization: {error}"),
    }
}
