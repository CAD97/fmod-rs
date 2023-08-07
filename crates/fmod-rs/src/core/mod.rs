#![doc = include_str!("README.md")]

/// Functionality not associated with a specific object.
pub mod common {
    pub mod debug;
    pub mod file;
    mod general;
    pub mod memory;
    mod mix;
    pub mod thread;

    pub use self::{general::*, mix::*};
}

pub mod effect;

fmod_class! {
    /// Management object from which all resources are created and played.
    ///
    /// Create with [`System::new`].
    class System = FMOD_SYSTEM;

    mod lifetime, device, setup, file, plugin, network, information, creation,
    runtime, recording, geometry, general;
}

fmod_class! {
    /// Container for [sample data](https://fmod.com/docs/2.02/api/glossary.html#sample-data) that can be played on a [Channel].
    ///
    /// Create with [`System::create_sound`] or [System::create_stream].
    class Sound = FMOD_SOUND;

    mod format, default, relationship, data, music, synchronization, general, ios;
}

fmod_class! {
    /// The shared APIs between [`Channel`] and [`ChannelGroup`].
    weak class ChannelControl = FMOD_CHANNELCONTROL;

    mod playback, volume, spatialization, panning, filtering, dsp, scheduling, general;
}

fmod_class! {
    /// A source of audio signal that connects to the [`ChannelGroup`] mixing hierarchy.
    ///
    /// Create with [System::play_sound] or [System::play_dsp].
    weak class Channel = FMOD_CHANNEL;

    mod playback, information, general;
}

fmod_class! {
    /// A submix in the mixing hierarchy akin to a bus that can contain both [`Channel`] and [`ChannelGroup`] objects.
    ///
    /// Create with [`System::create_channel_group`].
    class ChannelGroup = FMOD_CHANNELGROUP;

    mod channel, group, general;
}

fmod_class! {
    /// An interface that manages groups of [`Sound`]s.
    class SoundGroup = FMOD_SOUNDGROUP;

    mod group, sound, general;
}

fmod_class! {
    /// The Digital Signal Processor is one node within a graph that transforms input audio signals to an output stream.
    ///
    /// Create with [`System::create_dsp`], [`System::create_dsp_by_type`] or [`System::create_dsp_by_plugin`].
    class Dsp = FMOD_DSP;

    mod connections, parameters, channel, metering, processing, general, effect;
}

fmod_class! {
    /// An interface that manages Digital Signal Processor (DSP) Connections.
    ///
    /// # Safety
    ///
    /// Unlike most other handles in the FMOD API, DSP connections' lifetime is
    /// dynamically tied to the actual connection state of the DSPs, and it is
    /// UB to use a connection after it gets disconnected. Because of this, all
    /// APIs which return `DspConnection` return it by pointer, and it is up to
    /// you as the developer to ensure that the connection has not been removed
    /// when using it.
    weak class DspConnection = FMOD_DSPCONNECTION;

    mod mix, general;
}

fmod_class! {
    /// An interface that allows the setup and modification of geometry for occlusion.
    class Geometry = FMOD_GEOMETRY;

    mod polygons, spatialization, general;
}

fmod_class! {
    /// An interface that manages virtual 3D reverb spheres.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    class Reverb3d = FMOD_REVERB3D;

    mod general;
}

pub use self::common::*;
