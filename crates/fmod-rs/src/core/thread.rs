use fmod::{raw::*, *};

/// Specify the affinity, priority and stack size for all FMOD created
/// threads.
///
/// Affinity can be specified using one (or more) of the [ThreadAffinity]
/// constants or by providing the bits explicitly, i.e. (1<<3) for logical
/// core three (core affinity is zero based).  
/// See platform documentation for details on the available cores for a
/// given device.
///
/// Priority can be specified using one of the [ThreadPriority] constants or
/// by providing the value explicitly, i.e. (-2) for the lowest thread
/// priority on Windows.  
/// See platform documentation for details on the available priority values
/// for a given operating system.
///
/// Stack size can be specified explicitly, however for each thread you
/// should provide a size equal to or larger than the expected default or
/// risk causing a stack overflow at runtime.
///
/// # Safety
///
/// This function must be called before any FMOD [System] object is created.
pub unsafe fn set_attributes(
    kind: ThreadType,
    affinity: ThreadAffinity,
    priority: ThreadPriority,
    stack_size: ThreadStackSize,
) -> Result {
    fmod_try!(FMOD_Thread_SetAttributes(
        kind.into_raw(),
        affinity.into_raw(),
        priority.into_raw(),
        stack_size.into_raw()
    ));
    Ok(())
}
