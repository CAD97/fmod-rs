#[cfg_attr(feature = "unstable", feature(once_cell))]

macro_rules! opaque {
    ($($(#[$meta:meta])* class $Name:ident;)*) => {$(
        #[repr(C)]
        $(#[$meta])*
        pub struct $Name {
            _data: std::cell::Cell<[u8; 0]>,
            _marker: std::marker::PhantomData<(*mut u8, std::marker::PhantomPinned)>,
        }
    )*};
}

#[cfg(feature = "raw")]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        $(#[$meta])* pub $($tt)*
    };
}

#[cfg(not(feature = "raw"))]
macro_rules! raw {
    ($(#[$meta:meta])* pub $($tt:tt)*) => {
        #[allow(dead_code)]
        $(#[$meta])* pub(crate) $($tt)*
    };
}

extern crate self as fmod;

mod common;
mod core;
mod error;

pub use self::{common::*, core::*, error::*};

raw! {
    pub mod raw {
        #[doc(inline)]
        pub use fmod_core_sys::*;

        #[doc(inline)]
        #[cfg(feature = "studio")]
        pub use fmod_studio_sys::*;
    }
}

#[cfg(feature = "tracing")]
fn span() -> tracing::Span {
    tracing::info_span!(parent: None, "fmod")
}

#[cfg(feature = "tracing")]
fn memory_span() -> tracing::Span {
    tracing::debug_span!(parent: &crate::span(), "memory")
}

#[cfg(feature = "tracing")]
fn file_span() -> tracing::Span {
    tracing::debug_span!(parent: &crate::span(), "file")
}

#[cfg(feature = "tracing")]
fn codec_span() -> tracing::Span {
    tracing::debug_span!(parent: &crate::span(), "codec")
}
