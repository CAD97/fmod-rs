use {cfg_if::cfg_if, std::panic::UnwindSafe};

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

pub fn catch_user_unwind<F, R>(f: F) -> Option<R>
where
    F: UnwindSafe + FnOnce() -> R,
{
    match std::panic::catch_unwind(f) {
        Ok(x) => Some(x),
        Err(err) => {
            let callback = std::any::type_name::<F>();
            if let Some(e) = cool_asserts::get_panic_message(&err) {
                cfg_if! {
                    if #[cfg(feature = "tracing")] {
                        tracing::error!(
                            parent: crate::span(),
                            callback,
                            "FMOD.rs panicked in a callback: {e}",
                        );
                    } else {
                        eprintln!("FMOD.rs panicked in {callback}: {e}");
                    }
                }
            } else {
                cfg_if! {
                    if #[cfg(feature = "tracing")] {
                        tracing::error!(
                            parent: crate::span(),
                            callback,
                            "FMOD.rs panicked in a callback",
                        );
                    } else {
                        eprintln!("FMOD.rs panicked in {callback}: {e}");
                    }
                }
            }
            None
        },
    }
}
