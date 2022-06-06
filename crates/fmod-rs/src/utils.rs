use {
    fmod::{Error, Result},
    std::{
        borrow::Cow,
        ffi::CStr,
        mem::{self, MaybeUninit},
        os::raw::c_char,
        panic::UnwindSafe,
        ptr,
    },
};

/// Decode a UTF-16LE–encoded slice `v` into a `String`, replacing
/// invalid data with [the replacement character (`U+FFFD`)][U+FFFD].
///
/// Unlike [`from_utf8_lossy`] which returns a [`Cow<'a, str>`],
/// `from_utf16le_lossy` returns a `String` since the UTF-16 to UTF-8
/// conversion requires a memory allocation.
///
/// [`from_utf8_lossy`]: String::from_utf8_lossy
/// [`Cow<'a, str>`]: crate::borrow::Cow "borrow::Cow"
/// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,ignore
/// // 𝄞mus<invalid>ic<invalid>
/// let v = &[0x34, 0xD8, 0x1E, 0xDD, 0x6d, 0x00, 0x75, 0x00,
///           0x73, 0x00, 0x1E, 0xDD, 0x69, 0x00, 0x63, 0x00,
///           0x34, 0xD8];
///
/// assert_eq!(String::from("𝄞mus\u{FFFD}ic\u{FFFD}"),
///            String::from_utf16le_lossy(v));
/// ```
pub fn string_from_utf16le_lossy(v: &[u8]) -> String {
    match (cfg!(target_endian = "little"), unsafe {
        v.align_to::<u16>()
    }) {
        (true, (&[], v, &[])) => String::from_utf16_lossy(v),
        (true, (&[], v, &[_remainder])) => String::from_utf16_lossy(v) + "\u{FFFD}",
        _ => {
            let mut iter = v.chunks_exact(2);
            let string = char::decode_utf16(
                iter.by_ref()
                    .map(TryFrom::try_from)
                    .map(Result::unwrap)
                    .map(u16::from_le_bytes),
            )
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
            if iter.remainder().is_empty() {
                string
            } else {
                string + "\u{FFFD}"
            }
        },
    }
}

/// Decode a UTF-16BE–encoded slice `v` into a `String`, replacing
/// invalid data with [the replacement character (`U+FFFD`)][U+FFFD].
///
/// Unlike [`from_utf8_lossy`] which returns a [`Cow<'a, str>`],
/// `from_utf16le_lossy` returns a `String` since the UTF-16 to UTF-8
/// conversion requires a memory allocation.
///
/// [`from_utf8_lossy`]: String::from_utf8_lossy
/// [`Cow<'a, str>`]: crate::borrow::Cow "borrow::Cow"
/// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
///
/// # Examples
///
/// Basic usage:
///
/// ```rust,ignore
/// // 𝄞mus<invalid>ic<invalid>
/// let v = &[0xD8, 0x34, 0xDD, 0x1E, 0x00, 0x6d, 0x00, 0x75,
///           0x00, 0x73, 0xDD, 0x1E, 0x00, 0x69, 0x00, 0x63,
///           0xD8, 0x34];
///
/// assert_eq!(String::from("𝄞mus\u{FFFD}ic\u{FFFD}"),
///            String::from_utf16be_lossy(v));
/// ```
pub fn string_from_utf16be_lossy(v: &[u8]) -> String {
    match (cfg!(target_endian = "big"), unsafe { v.align_to::<u16>() }) {
        (true, (&[], v, &[])) => String::from_utf16_lossy(v),
        (true, (&[], v, &[_remainder])) => String::from_utf16_lossy(v) + "\u{FFFD}",
        _ => {
            let mut iter = v.chunks_exact(2);
            let string = char::decode_utf16(
                iter.by_ref()
                    .map(TryFrom::try_from)
                    .map(Result::unwrap)
                    .map(u16::from_be_bytes),
            )
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect();
            if iter.remainder().is_empty() {
                string
            } else {
                string + "\u{FFFD}"
            }
        },
    }
}

pub fn string_extend_utf8_lossy(s: &mut String, mut v: &[u8]) {
    s.reserve(v.len());
    loop {
        match std::str::from_utf8(v) {
            Ok(rest) => {
                s.push_str(rest);
                break;
            },
            Err(err) => {
                let valid_up_to = err.valid_up_to();
                let valid =
                    unsafe { std::str::from_utf8_unchecked(v.get_unchecked(..valid_up_to)) };
                s.push_str(valid);
                s.push(char::REPLACEMENT_CHARACTER);
                match err.error_len() {
                    None => break,
                    Some(len) => v = unsafe { v.get_unchecked(valid_up_to + len..) },
                }
            },
        }
    }
}

/// Decode [Simple Binary Coded Decimal](https://en.wikipedia.org/wiki/Binary-coded_decimal).
#[allow(clippy::erasing_op, clippy::identity_op)]
pub const fn decode_sbcd_u8(encoded: u8) -> u8 {
    const MASK: u8 = 0xF;
    const SHIFT: u8 = 4;
    000 + (((encoded >> (SHIFT * 0)) & MASK) * 1) //-
        + (((encoded >> (SHIFT * 1)) & MASK) * 10)
}

/// Decode [Simple Binary Coded Decimal](https://en.wikipedia.org/wiki/Binary-coded_decimal).
#[allow(clippy::erasing_op, clippy::identity_op)]
pub const fn decode_sbcd_u16(encoded: u16) -> u16 {
    const MASK: u16 = 0xF;
    const SHIFT: u8 = 4;
    000 + (((encoded >> (SHIFT * 0)) & MASK) * 1)
        + (((encoded >> (SHIFT * 1)) & MASK) * 10)
        + (((encoded >> (SHIFT * 2)) & MASK) * 100)
        + (((encoded >> (SHIFT * 3)) & MASK) * 1000)
}

pub fn catch_user_unwind<F, R>(f: F) -> Option<R>
where
    F: UnwindSafe + FnOnce() -> R,
{
    match std::panic::catch_unwind(f) {
        Ok(x) => Some(x),
        Err(err) => {
            let callback = std::any::type_name::<F>();
            if let Some(e) = cool_asserts::get_panic_message(&err) {
                whoops!(
                    trace(callback, "FMOD.rs panicked in a callback: {e}"),
                    stderr("FMOD.rs panicked in {callback}: {e}")
                );
            } else {
                whoops!(
                    trace(callback, "FMOD.rs panicked in a callback"),
                    stderr("FMOD.rs panicked in {callback}"),
                );
            }
            None
        },
    }
}

pub unsafe fn str_from_nonnull_unchecked<'a>(ptr: ptr::NonNull<c_char>) -> &'a str {
    CStr::from_ptr(ptr.as_ptr()).to_str().unwrap_unchecked()
}

pub unsafe fn fmod_get_string(
    buf: &mut String,
    mut retry: impl FnMut(&mut [MaybeUninit<u8>]) -> Result,
) -> Result {
    buf.clear();

    // multiple_system.cpp uses a 256 byte buffer for System::get_driver_name
    // follow that example and try a 256 byte buffer before heap reallocation
    const STACK_BUFFER_SIZE: usize = 256;

    // only use the stack buffer if the heap buffer isn't that large already
    if buf.capacity() < STACK_BUFFER_SIZE {
        let mut stack_buf: [MaybeUninit<u8>; STACK_BUFFER_SIZE] =
            MaybeUninit::uninit().assume_init();

        // first try with stack buffer
        match retry(&mut stack_buf) {
            Ok(()) => {
                let cstr = CStr::from_ptr(stack_buf.as_ptr().cast());
                string_extend_utf8_lossy(buf, cstr.to_bytes());
                return Ok(());
            },
            Err(Error::Truncated) => (), // continue
            Err(err) => return Err(err),
        }

        // keep trying with larger buffers
        buf.reserve(STACK_BUFFER_SIZE * 2);
    }

    let buf = buf.as_mut_vec();
    loop {
        // try again
        match retry(buf.spare_capacity_mut()) {
            Ok(()) => break,
            Err(Error::Truncated) => {
                // keep trying with larger buffers
                buf.reserve(buf.capacity() * 2);
            },
            Err(err) => return Err(err),
        }
    }

    // now we need to set the vector length and verify proper UTF-8
    buf.set_len(CStr::from_ptr(buf.as_ptr().cast()).to_bytes().len());
    match String::from_utf8_lossy(buf) {
        Cow::Borrowed(_) => (), // valid, leave in string buf
        Cow::Owned(fix) => {
            // swap in the fixed UTF-8
            mem::swap(buf, &mut fix.into_bytes());
        },
    }

    Ok(())
}
