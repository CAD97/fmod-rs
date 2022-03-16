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
    (pub $($tt:tt)*) => {
        pub(crate) $($tt)*
    };
}

use {once_cell::sync::Lazy, std::sync::atomic::AtomicBool};

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
static SPAN: Lazy<tracing::Span> = Lazy::new(|| tracing::info_span!(parent: None, "FMOD"));
static CREATE_ONCE: AtomicBool = AtomicBool::new(false);
