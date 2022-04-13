#![cfg_attr(feature = "unstable", feature(extern_types))]

#[macro_use]
mod macros;

extern crate self as fmod;

mod common;
mod core;
mod error;
mod handle;
pub(crate) mod utils;

pub use self::{common::*, core::*, error::*, handle::*};

raw! {
    pub mod raw {
        #[doc(inline)]
        pub use fmod_core_sys::*;

        #[doc(inline)]
        #[cfg(feature = "studio")]
        pub use fmod_studio_sys::*;
    }
}

/// Current FMOD version number.
pub const VERSION: Version = Version::from_raw(raw::FMOD_VERSION);

#[cfg(feature = "tracing")]
fn span() -> &'static tracing::Span {
    use once_cell::sync::OnceCell;
    static ONCE: OnceCell<tracing::Span> = OnceCell::new();
    ONCE.get_or_init(|| tracing::error_span!(target: "fmod", parent: None, "fmod"))
}
