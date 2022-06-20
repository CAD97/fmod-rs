use {
    crate::utils::{catch_user_unwind, str_from_nonnull_unchecked},
    fmod::{raw::*, *},
    std::{
        os::raw::{c_char, c_int},
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
/// <pre class="ignore" style="white-space:normal;font:inherit;">
/// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
/// the <code>fmod_debug_is_tracing</code> feature flag is set. Manually
/// initializing FMOD debugging will override this behavior.
/// </pre>
pub fn initialize(flags: DebugFlags) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();
    // suppress default debug init
    DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

    fmod_try!(FMOD_Debug_Initialize(
        flags.into_raw(),
        DebugMode::Tty.into_raw(),
        None,
        ptr::null()
    ));
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
/// <pre class="ignore" style="white-space:normal;font:inherit;">
/// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
/// the <code>fmod_debug_is_tracing</code> feature flag is set. Manually
/// initializing FMOD debugging will override this behavior.
/// </pre>
pub fn initialize_callback<D: FmodDebug>(flags: DebugFlags) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();
    // suppress default debug init
    DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

    fmod_try!(FMOD_Debug_Initialize(
        flags.into_raw(),
        DebugMode::Callback.into_raw(),
        Some(callback::<D>),
        ptr::null()
    ));
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
/// - [DebugFlags::LevelLog] produces informational, warning and error
///   messages.
/// - [DebugFlags::LevelWarning] produces warnings and error messages.
/// - [DebugFlags::LevelError] produces error messages only.
///
/// <pre class="ignore" style="white-space:normal;font:inherit;">
/// FMOD.rs automatically initializes FMOD's debug logs to go to tracing if
/// the <code>fmod_debug_is_tracing</code> feature flag is set. Manually
/// initializing FMOD debugging will override this behavior.
/// </pre>
pub fn initialize_file(flags: DebugFlags, file: &CStr8) -> Result {
    // prevent racing System init
    let _lock = GLOBAL_SYSTEM_STATE.read();
    // suppress default debug init
    DEBUG_LAYER_INITIALIZED.store(true, Ordering::Release);

    fmod_try!(FMOD_Debug_Initialize(
        flags.into_raw(),
        DebugMode::File.into_raw(),
        None,
        file.as_ptr() as _
    ));
    Ok(())
}

#[cfg(feature = "tracing")]
/// An `FmodDebug` sink that hooks up FMOD debug messages into [tracing].
///
/// Mapping:
/// - [DebugFlags::TypeMemory]   ~ [Level::TRACE], target `fmod::memory`
/// - [DebugFlags::TypeFile]     ~ [Level::TRACE], target `fmod::file`
/// - [DebugFlags::TypeCodec]    ~ [Level::TRACE], target `fmod::codec`
/// - [DebugFlags::TypeTrace]    ~ [Level::TRACE], target `fmod::trace`
/// - [DebugFlags::LevelLog]     ~ [Level::INFO],  target `fmod`
/// - [DebugFlags::LevelWarning] ~ [Level::WARN],  target `fmod`
/// - [DebugFlags::LevelError]   ~ [Level::ERROR], target `fmod`
///
/// All events are emitted under a orphaned (no parent) error span with name
/// and target `fmod` which is never explicitly entered nor exited.
///
/// `file`, `line`, `func`, and `message` are fed as-is as event fields to
/// the tracing backend.
///
/// [Level::TRACE]: tracing::Level::TRACE
/// [Level::DEBUG]: tracing::Level::DEBUG
/// [Level::INFO]: tracing::Level::INFO
/// [Level::WARN]: tracing::Level::WARN
/// [Level::ERROR]: tracing::Level::ERROR
pub enum FmodDebugTracing {}

#[cfg(feature = "tracing")]
impl FmodDebug for FmodDebugTracing {
    fn log(
        flags: DebugFlags,
        file: Option<&str>,
        line: i32,
        func: Option<&str>,
        message: Option<&str>,
    ) -> Result<()> {
        if flags.is_set(DebugFlags::TypeMemory) {
            tracing::trace!(target: "fmod::memory", parent: crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::TypeFile) {
            tracing::trace!(target: "fmod::file", parent: crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::TypeCodec) {
            tracing::trace!(target: "fmod::codec", parent: crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::TypeTrace) {
            tracing::trace!(target: "fmod::trace", parent: crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::LevelLog) {
            tracing::info!(target: "fmod", parent: crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::LevelWarning) {
            tracing::warn!(target: "fmod", parent: crate::span(), file, line, func, message)
        } else if flags.is_set(DebugFlags::LevelError) {
            tracing::error!(target: "fmod", parent: crate::span(), file, line, func, message)
        } else {
            tracing::error!(
                parent: crate::span(),
                debug_flags = ?flags,
                file,
                line,
                func,
                message,
            );
            return Err(Error::InternalRs);
        };
        Ok(())
    }
}

#[cfg(feature = "tracing")]
impl FmodDebugTracing {
    /// Create [DebugFlags] enabling only debug logging which is enabled by
    /// the current default tracing subscriber.
    ///
    /// This relies on the current contextual subscriber being the actual
    /// subscriber which we eventually dispatch to; if the contextual
    /// subscriber is later changed to a subscriber that enables more than
    /// the original did, FMOD will not be emitting the events which the new
    /// subscriber but not the old subscriber are interested in.
    ///
    /// Filtering out logging on the FMOD side is a desirable default, but
    /// if you change the contextual subscriber, you should reinitialize the
    /// FMOD debug layer or just originally initialize it with
    /// DebugFlags::All and let dynamic filtering take care of it.
    ///
    /// FMOD's default filter is [DebugFlags::LevelLog], which is equivalent
    /// to a tracing filter of `fmod=INFO` when using `FmodDebugTracing`.
    /// Enabling the `fmod::memory`/`fmod::file`/`fmod::codec` targets at
    /// trace level should only be done when debugging specific issues that
    /// require tracing that area's execution; these are truly verbose trace
    /// level logging targets.
    pub fn ideal_debug_flags() -> DebugFlags {
        use tracing::{enabled, Level};
        let mut debug_flags = DebugFlags::LevelNone;

        if enabled!(target: "fmod", Level::ERROR, { file, line, func, message }) {
            debug_flags = DebugFlags::LevelError;
        }
        if enabled!(target: "fmod", Level::WARN, { file, line, func, message }) {
            debug_flags = DebugFlags::LevelWarning;
        }
        if enabled!(target: "fmod", Level::INFO, { file, line, func, message }) {
            debug_flags = DebugFlags::LevelLog;
        }
        if enabled!(target: "fmod::trace", Level::TRACE, { file, line, func, message }) {
            debug_flags |= DebugFlags::TypeTrace;
        }
        if enabled!(target: "fmod::memory", Level::TRACE, { file, line, func, message }) {
            debug_flags |= DebugFlags::TypeMemory;
        }
        if enabled!(target: "fmod::file", Level::TRACE, { file, line, func, message }) {
            debug_flags |= DebugFlags::TypeFile;
        }
        if enabled!(target: "fmod::codec", Level::TRACE, { file, line, func, message }) {
            debug_flags |= DebugFlags::TypeCodec;
        }

        debug_flags
    }
}

#[cfg(feature = "tracing")]
fn handle_init_failure(error: Error) {
    match error {
        Error::Unsupported => {
            tracing::info!(parent: crate::span(), "FMOD debug disabled");
        },
        error => {
            whoops!("Error during FMOD debug initialization: {error}");
        },
    }
}

#[cfg(feature = "tracing")]
/// Initialize the FMOD debug layer to write to the contextual tracing
/// subscriber via [`FmodDebugTracing`].
///
/// If FMOD debugging is statically disabled, logs this via `info!`.
///
/// If the `fmod_debug_is_tracing` feature flag is set, this is done
/// automatially for you when an FMOD System is first created. This is only
/// required when changing the contextual subscriber (see
/// [`FmodDebugTracing::ideal_debug_flags`] for why) or if changing the
/// FMOD debug layer's state at runtime.
pub fn initialize_tracing() {
    match initialize_callback::<FmodDebugTracing>(FmodDebugTracing::ideal_debug_flags()) {
        Ok(()) => (),
        Err(error) => handle_init_failure(error),
    }
}

/// Initialize the FMOD debug layer to tracing only if it hasn't been
/// already initialized and skipping the global read lock against racing
/// system init. Called as part of system init.
pub(crate) unsafe fn initialize_default() {
    #[cfg(feature = "fmod_debug_is_tracing")]
    {
        if DEBUG_LAYER_INITIALIZED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let flags = FmodDebugTracing::ideal_debug_flags();
            let error = FMOD_Debug_Initialize(
                flags.into_raw(),
                DebugMode::Callback.into_raw(),
                Some(callback::<FmodDebugTracing>),
                ptr::null(),
            );
            match Error::from_raw(error) {
                Ok(()) => (),
                Err(error) => handle_init_failure(error),
            }
        }
    }
}
