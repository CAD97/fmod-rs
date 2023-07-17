#![doc = include_str!("README.md")]

/// Functionality not associated with a specific object.
pub mod common {
    pub mod debug;
    pub mod file;
    mod general;
    pub mod memory;
    pub mod thread;

    pub use self::general::*;
}

opaque! {
    /// Management object from which all resources are created and played.
    ///
    /// Create with [`System::new`].
    class System = FMOD_SYSTEM;

    mod lifetime, device, setup, file, plugin, network, information, creation,
    runtime, recording, geometry, general;
}

opaque! {
    /// Container for [sample data](https://fmod.com/docs/2.02/api/glossary.html#sample-data) that can be played on a [Channel].
    ///
    /// Create with [`System::create_sound`] or [System::create_stream].
    class Sound = FMOD_SOUND;

    mod;
}

opaque! {
    /// The shared APIs between [`Channel`] and [`ChannelGroup`].
    weak class ChannelControl = FMOD_CHANNELCONTROL;

    mod;
}

opaque! {
    /// A source of audio signal that connects to the [`ChannelGroup`] mixing hierarchy.
    ///
    /// Create with [System::play_sound] or [System::play_dsp].
    weak class Channel = FMOD_CHANNEL;

    mod;
}

opaque! {
    /// A submix in the mixing hierarchy akin to a bus that can contain both [`Channel`] and [`ChannelGroup`] objects.
    ///
    /// Create with [`System::create_channel_group`].
    class ChannelGroup = FMOD_CHANNELGROUP;

    mod;
}

opaque! {
    /// An interface that manages groups of [`Sound`]s.
    class SoundGroup = FMOD_SOUNDGROUP;

    mod;
}

opaque! {
    /// The Digital Signal Processor is one node within a graph that transforms input audio signals to an output stream.
    ///
    /// Create with [`System::create_dsp`], [`System::create_dsp_by_type`] or [`System::create_dsp_by_plugin`].
    class Dsp = FMOD_DSP;

    mod;
}

opaque! {
    /// An interface that manages Digital Signal Processor (DSP) Connections.
    weak class DspConnection = FMOD_DSPCONNECTION;

    mod;
}

opaque! {
    /// An interface that allows the setup and modification of geometry for occlusion.
    class Geometry = FMOD_GEOMETRY;

    mod;
}

opaque! {
    /// An interface that manages virtual 3D reverb spheres.
    ///
    /// See the 3D Reverb guide for more information.
    class Reverb3d = FMOD_REVERB3D;

    mod;
}

// pub mod channel;
// pub mod channel_control;
// pub mod channel_group;
// pub mod dsp;
// pub mod dsp_connection;
mod effect;
mod ex;
// pub mod geometry;
mod ios;
// pub mod reverb3d;
// pub mod sound;
// pub mod sound_group;

pub use self::{common::*, effect::*, ex::*, ios::*};
