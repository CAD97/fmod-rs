#![doc = include_str!("README.md")]

// !!! module order is important for documentation listing order !!!

pub mod common {
    //! Functionality not associated with a specific object.

    pub mod debug;
    pub mod file;
    mod general;
    pub mod memory;
    pub mod thread;

    pub use self::general::*;
}

#[rustfmt::skip]
pub mod system {
    //! Management object from which all resources are created and played.

    use fmod::{raw::*, *};

    opaque! {
        /// Management object from which all resources are created and played.
        ///
        /// Create with [`System::new`].
        class System = FMOD_SYSTEM, FMOD_System_* (System::raw_release);
    }

    mod lifetime;
    mod device;
    mod setup;
    mod file;
    mod plugin;
    mod network;
    mod information;
    mod creation;
    mod runtime;
    mod recording;
    mod geometry;
    mod general;

    pub use self::{
        lifetime::*, device::*, setup::*, file::*, plugin::*, network::*, information::*,
        creation::*, runtime::*, recording::*, geometry::*, general::*,
    };
}

pub mod channel;
pub mod channel_control;
pub mod channel_group;
pub mod dsp;
pub mod dsp_connection;
mod effect;
mod ex;
pub mod geometry;
mod ios;
pub mod reverb_3d;
pub mod sound;
pub mod sound_group;

pub use self::{
    channel::{Channel, *},
    channel_control::{ChannelControl, *},
    channel_group::{ChannelGroup, *},
    common::*,
    dsp::{Dsp, *},
    dsp_connection::{DspConnection, *},
    effect::*,
    ex::*,
    geometry::{Geometry, *},
    ios::*,
    reverb_3d::{Reverb3d, *},
    sound::{Sound, *},
    sound_group::{SoundGroup, *},
    system::{System, *},
};
