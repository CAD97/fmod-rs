#![cfg_attr(feature = "unstable", feature(extern_types, doc_auto_cfg, doc_cfg))]
#![warn(rust_2018_idioms)] // lol, this reads backwards

//! # FMOD.rs
//!
//! FMOD.rs provides Rust bindings to the [FMOD adaptive audio solution][FMOD].
//!
//! [FMOD]: https://fmod.com/
//!
#![cfg_attr(
    feature = "studio",
    doc = "\
If you are integrating FMOD Studio into your game, the best starting point is
the [studio] module documentation, which serves as an API guide alongside an API
reference. The Studio API plays back content created within the FMOD Studio
authoring too. Studio's data-driven approach means audio behaviors remain easily
accessible and editable to sound designers.

If your project has custom requirements that go beyond what the FMOD Studio API
offers, the 
"
)]
#![cfg_attr(not(feature = "studio"), doc = " The")]
//! FMOD Core API provides fast and flexible access to low-level audio
//! primitives. Start with the [core] module documentation, which serves as an
//! API guide alongside an API reference.
//!
#![cfg_attr(
    feature = "fsbank",
    doc = "\
Additionally, to integrate the creation of compressed assets using the FSB file
format into your tools and development pipelines, use the [fsbank] module.
"
)]
//!
//! Whether you're building with Studio or Core it's important to consider your
//! target platform and any specific functionality, compatibility, and
//! requirements it may have. You can see details documented in the [platform]
//! module.
//!
#![doc = ::document_features::document_features!()]

#[macro_use]
mod macros;

extern crate self as fmod;

pub mod core;
#[cfg(doc)]
pub mod platform;
#[cfg(feature = "studio")]
pub mod studio;

mod error;
mod handle;
pub(crate) mod utils;

#[doc(no_inline)]
pub use self::core::*;

#[doc(inline)]
pub use self::{error::*, handle::*};

#[doc(hidden)]
pub use cstr::cstr as _cstr;

raw! {
    /// Raw access to the FMOD C API.
    pub mod raw {
        #[doc(inline)]
        pub use fmod_core_sys::*;

        #[doc(inline)]
        #[cfg(feature = "studio")]
        pub use fmod_studio_sys::*;
    }
}

/// A macro for getting `&'static CStr` from literal or identifier.
///
/// This macro checks whether the given literal is valid for `CStr` at compile
/// time, and returns a constant reference of `CStr`.
///
/// # Examples
///
/// ```rust,ignore
/// # use {fmod::cstr, std::ffi::CStr};
/// let a = cstr!(b"hello\xff");
/// let b = cstr!("hello");
/// let c = cstr!(hello);
///
/// assert_eq!(a, CStr::from_bytes_with_nul(b"hello\xff\0").unwrap());
/// assert_eq!(b, CStr::from_bytes_with_nul(b"hello\0").unwrap());
/// assert_eq!(c, CStr::from_bytes_with_nul(b"hello\0").unwrap());
/// ```
#[macro_export]
macro_rules! cstr {
    ($x:literal) => {
        $crate::_cstr! { $x }
    };
    ($x:ident) => {
        $crate::_cstr! { $x }
    };
}

/// Current FMOD version number.
pub const VERSION: Version = Version::from_raw(raw::FMOD_VERSION);

#[cfg(feature = "tracing")]
fn span() -> &'static tracing::Span {
    use once_cell::sync::OnceCell;
    static ONCE: OnceCell<tracing::Span> = OnceCell::new();
    ONCE.get_or_init(|| tracing::error_span!(target: "fmod", parent: None, "fmod"))
}
