#![allow(nonstandard_style, unused_parens, clippy::needless_return)]
#![allow(clippy::missing_safety_doc, clippy::unnecessary_cast)]

use fmod_core_sys::*;

include!(concat!(env!("OUT_DIR"), "/fmod_studio.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_studio_common.rs"));
