#![cfg_attr(feature = "unstable", feature(const_weak_new, extern_types, once_cell))]

#[macro_use]
mod macros;

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
    tracing::error_span!(target: "fmod", parent: None, "fmod")
}
