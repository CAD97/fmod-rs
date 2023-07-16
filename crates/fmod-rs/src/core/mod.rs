#![doc = include_str!("README.md")]

// !!! module order is important for documentation listing order !!!

#[rustfmt::skip]
mod system {
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
    mod ex;

    pub use self::{
        lifetime::*, device::*, setup::*, file::*, plugin::*, network::*, information::*,
        creation::*, runtime::*, recording::*, geometry::*, general::*, ex::*,
    };
}

mod channel;
mod channel_control;
mod channel_group;
mod dsp;
mod dsp_connection;
mod effect;
mod ex;
mod general;
mod geometry;
mod ios;
mod reverb_3d;
mod sound;
mod sound_group;

pub mod debug;
pub mod file;
pub mod memory;
pub mod thread;

pub use self::{
    channel::*, channel_control::*, channel_group::*, dsp::*, dsp_connection::*, effect::*, ex::*,
    general::*, geometry::*, ios::*, reverb_3d::*, sound::*, sound_group::*, system::*,
};
