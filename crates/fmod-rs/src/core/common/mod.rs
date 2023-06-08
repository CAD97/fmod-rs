mod enums;
mod flags;
mod structs;

pub use self::{enums::*, flags::*, structs::*};

use fmod::raw::*;

/// Maximum number of channels per frame of audio supported by audio files,
/// buffers, connections and DSPs.
pub const MAX_CHANNEL_WIDTH: usize = FMOD_MAX_CHANNEL_WIDTH as usize;

/// Maximum number of listeners supported.
pub const MAX_LISTENERS: usize = FMOD_MAX_LISTENERS as usize;

/// Maximum number of System objects allowed.
pub const MAX_SYSTEMS: usize = FMOD_MAX_SYSTEMS as usize;
