mod enums;
mod flags;
mod structs;

pub use self::{enums::*, flags::*, structs::*};

use fmod::raw::*;

/// Maximum number of channels per frame of audio supported by audio files,
/// buffers, connections and DSPs.
pub const MAX_CHANNEL_WIDTH: i32 = FMOD_MAX_CHANNEL_WIDTH as i32;

/// Maximum number of listeners supported.
pub const MAX_LISTENERS: i32 = FMOD_MAX_LISTENERS as i32;

/// Maximum number of System objects allowed.
pub const MAX_SYSTEMS: i32 = FMOD_MAX_SYSTEMS as i32;
