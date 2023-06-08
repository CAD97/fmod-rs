use {
    crate::utils::{catch_user_unwind, str_from_nonnull_unchecked},
    fmod::{raw::*, *},
    std::{
        ffi::{c_char, c_int},
        ptr,
        sync::atomic::{AtomicBool, Ordering},
    },
};

/// Callback for debug messages when using the logging version of FMOD.
///
/// This callback will fire directly from the log line, as such it can be
/// from any thread.
pub trait FmodDebug {
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
    ) -> Result<()>;
}

unsafe extern "system" fn callback<D: FmodDebug>(
    flags: FMOD_DEBUG_FLAGS,
    file: *const c_char,
    line: c_int,
    func: *const c_char,
    msg: *const c_char,
) -> FMOD_RESULT {
    catch_user_unwind(|| {
        let flags = DebugFlags::from_raw(flags);
        // SAFETY: these strings are produced directly by FMOD, so they
        // should *actually* be guaranteed to be UTF-8 like FMOD claims.
        let file = ptr::NonNull::new(file as *mut _)
            .map(|x| str_from_nonnull_unchecked(x))
            .map(str::trim_end);
        let func = ptr::NonNull::new(func as *mut _)
            .map(|x| str_from_nonnull_unchecked(x))
            .map(str::trim_end);
        let message = ptr::NonNull::new(msg as *mut _)
            .map(|x| str_from_nonnull_unchecked(x))
            .map(str::trim_end);
        D::log(flags, file, line, func, message)
    })
    .unwrap_or(Err(Error::InternalRs))
    .map_or_else(Error::into_raw, |()| FMOD_OK)
}

static DEBUG_LAYER_INITIALIZED: AtomicBool = AtomicBool::new(false);

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
    // suppress default debug init
    DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

    ffi!(FMOD_Debug_Initialize(
        flags.into_raw(),
        DebugMode::Tty.into_raw(),
        None,
        ptr::null()
    ))?;
    Ok(())
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
pub fn initialize_callback<D: FmodDebug>(flags: DebugFlags) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();
    // suppress default debug init
    DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

    ffi!(FMOD_Debug_Initialize(
        flags.into_raw(),
        DebugMode::Callback.into_raw(),
        Some(callback::<D>),
        ptr::null()
    ))?;
    Ok(())
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
    // suppress default debug init
    DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

    ffi!(FMOD_Debug_Initialize(
        flags.into_raw(),
        DebugMode::File.into_raw(),
        None,
        file.as_ptr() as _
    ))?;
    Ok(())
}

#[cfg(feature = "log")]
/// An `FmodDebug` sink that hooks up FMOD debug messages into [log].
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
/// Additionally, FMOD.rs traces handle creation/release in target fmod::handle.
pub enum FmodDebugLog {}

#[cfg(feature = "log")]
impl FmodDebug for FmodDebugLog {
    fn log(
        flags: DebugFlags,
        file: Option<&str>,
        line: i32,
        func: Option<&str>,
        message: Option<&str>,
    ) -> Result<()> {
        let mut log = log::Record::builder();
        let message = message.unwrap_or_default();
        log.file(file).line(Some(line as u32)).module_path(func);
        let res = if flags.is_set(DebugFlags::TypeMemory) {
            log.level(log::Level::Error).target("fmod::memory");
            Ok(())
        } else if flags.is_set(DebugFlags::TypeFile) {
            log.level(log::Level::Error).target("fmod::file");
            Ok(())
        } else if flags.is_set(DebugFlags::TypeCodec) {
            log.level(log::Level::Error).target("fmod::codec");
            Ok(())
        } else if flags.is_set(DebugFlags::TypeTrace) {
            log.level(log::Level::Trace).target("fmod::trace");
            Ok(())
        } else if flags.is_set(DebugFlags::LevelLog) {
            log.level(log::Level::Info).target("fmod");
            Ok(())
        } else if flags.is_set(DebugFlags::LevelWarning) {
            log.level(log::Level::Warn).target("fmod");
            Ok(())
        } else if flags.is_set(DebugFlags::LevelError) {
            log.level(log::Level::Error).target("fmod");
            Ok(())
        } else {
            log.level(log::Level::Error).target("fmod");
            log::error!("FMOD debug message with unknown flags: {:?}", flags);
            Err(Error::InternalRs)
        };
        log::logger().log(&log.args(format_args!("{message}")).build());
        res
    }
}

#[cfg(feature = "log")]
impl FmodDebugLog {
    /// Create [DebugFlags] enabling only debug logging which is enabled by
    /// the current logger.
    ///
    /// FMOD's default filter is [DebugFlags::LevelLog], which is equivalent
    /// to an env filter of `fmod=INFO` when using `FmodDebugLog`.
    /// Enabling the `fmod::memory`/`fmod::file`/`fmod::codec` targets at
    /// trace level should only be done when debugging specific issues that
    /// require tracing that area's execution; these are truly verbose trace
    /// level logging targets.
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

#[cfg(feature = "log")]
/// Initialize the FMOD debug layer to write to the global logger via
/// [`FmodDebugLog`].
///
/// If FMOD debugging is statically disabled, logs this via `info!`.
///
/// This is done automatically for you when an FMOD System is first created.
/// This is only required if changing log filtering or the FMOD debug layer's
/// state at runtime.
pub fn initialize_log() {
    match initialize_callback::<FmodDebugLog>(FmodDebugLog::ideal_debug_flags()) {
        Ok(()) => (),
        Err(error) => handle_init_failure(error),
    }
}

/// Initialize the FMOD debug layer only if it hasn't been already initialized
/// and skipping the global read lock against racing system init.
/// Called as part of system init.
pub(crate) unsafe fn initialize_default() {
    #[cfg(feature = "log")]
    {
        if !DEBUG_LAYER_INITIALIZED.swap(true, Ordering::AcqRel) {
            let flags = FmodDebugLog::ideal_debug_flags();
            let result = ffi!(FMOD_Debug_Initialize(
                flags.into_raw(),
                DebugMode::Callback.into_raw(),
                Some(callback::<FmodDebugLog>),
                ptr::null(),
            ));
            match result {
                Ok(()) => (),
                Err(error) => handle_init_failure(error),
            }
        }
    }
}
