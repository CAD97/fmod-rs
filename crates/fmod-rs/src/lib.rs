#![cfg_attr(
    feature = "unstable",
    feature(doc_auto_cfg, doc_cfg, read_buf, extern_types)
)]
#![allow(rustdoc::broken_intra_doc_links)] // TODO: remove once more items exist
#![allow(clippy::unit_arg)]
#![allow(clippy::missing_safety_doc)] // TODO: allow on individual items

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
#[cfg(feature = "fsbank")]
pub mod fsbank;
#[cfg(doc)]
#[cfg_attr(feature = "unstable", doc(cfg(doc)))]
pub mod platform;
#[cfg(feature = "studio")]
pub mod studio;

mod error;
mod handle;
pub(crate) mod utils;

#[doc(no_inline)]
pub use {
    self::core::*,
    cstr8::{cstr8, CStr8},
};

#[doc(inline)]
pub use self::{error::*, handle::*};

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
