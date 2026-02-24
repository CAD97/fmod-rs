#![allow(nonstandard_style, unused_parens, clippy::needless_return)]
#![allow(clippy::missing_safety_doc, clippy::unnecessary_cast)]

include!(concat!(env!("OUT_DIR"), "/fmod_codec.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_common.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_dsp_effects.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_dsp.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_errors.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod_output.rs"));
include!(concat!(env!("OUT_DIR"), "/fmod.rs"));

#[cfg(target_vendor = "uwp")]
include!(concat!(env!("OUT_DIR"), "/fmod_uwp.rs"));

#[cfg(all(target_vendor = "apple", not(target_os = "macos")))]
include!(concat!(env!("OUT_DIR"), "/fmod_ios.rs"));
