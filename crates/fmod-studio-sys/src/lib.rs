#![allow(nonstandard_style, unused_parens, clippy::needless_return)]
#![allow(clippy::missing_safety_doc, clippy::unnecessary_cast)]
#![cfg_attr(docsrs, doc = "This crate is intentionally empty on doc.rs.")]

use fmod_core_sys::*;

include!(concat!(env!("OUT_DIR"), "/fmod_studio.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_studio_common.rs"));
