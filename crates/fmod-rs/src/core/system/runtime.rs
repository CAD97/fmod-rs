use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Runtime control.
impl System {
    /// Sets the position, velocity and orientation of the specified 3D sound
    /// listener.
    ///
    /// The `forward` and `up` vectors must be perpendicular and be of unit
    /// length (magnitude of each vector should be 1).
    ///
    /// Vectors must be provided in the correct [handedness].
    ///
    /// [handedness]: https://fmod.com/resources/documentation-api?version=2.02&page=glossary.html#handedness
    ///
    /// For velocity, remember to use units per **second**, and not units per
    /// frame. This is a common mistake and will make the doppler effect sound
    /// wrong if velocity is based on movement per frame rather than a fixed
    /// time period.  
    /// If velocity per frame is calculated, it can be converted to velocity per
    /// second by dividing it by the time taken between frames as a fraction of
    /// a second.  
    /// i.e.
    ///
    /// ```rust
    /// # let [position_currentframe, position_lastframe, time_taken_since_last_frame_in_seconds] = [1; 3];
    /// let velocity_units_per_second =
    ///     (position_currentframe - position_lastframe)
    ///         / time_taken_since_last_frame_in_seconds;
    /// ```
    ///
    /// At 60fps, `time_taken_since_last_frame_in_seconds` will be 1/60.
    ///
    /// Users of the Studio API should call
    /// [studio::System::set_listener_attributes] instead of this function.
    pub fn set_3d_listener_attributes(
        &self,
        listener: i32,
        attributes: ListenerAttributes3d,
    ) -> Result {
        ffi!(FMOD_System_Set3DListenerAttributes(
            self.as_raw(),
            listener,
            attributes.pos.as_raw(),
            attributes.vel.as_raw(),
            attributes.forward.as_raw(),
            attributes.up.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the position, velocity and orientation of the specified 3D sound listener.
    ///
    /// Users of the Studio API should call
    /// [studio::System::get_listener_attributes] instead of this function.
    pub fn get_3d_listener_attributes(&self, listener: i32) -> Result<ListenerAttributes3d> {
        let mut attributes = ListenerAttributes3d::default();
        ffi!(FMOD_System_Get3DListenerAttributes(
            self.as_raw(),
            listener,
            attributes.pos.as_raw_mut(),
            attributes.vel.as_raw_mut(),
            attributes.forward.as_raw_mut(),
            attributes.up.as_raw_mut(),
        ))?;
        Ok(attributes)
    }

    /// Sets parameters for the global reverb environment.
    ///
    /// To assist in defining reverb properties there are several presets
    /// available, see [ReverbProperties]' associated constants.
    ///
    /// When using each instance for the first time, FMOD will create a physical
    /// SFX reverb DSP unit that takes up several hundred kilobytes of memory
    /// and some CPU.
    pub fn set_reverb_properties(
        &self,
        instance: i32,
        properties: Option<&ReverbProperties>,
    ) -> Result {
        ffi!(FMOD_System_SetReverbProperties(
            self.as_raw(),
            instance,
            properties.map_or(ptr::null(), |x| x.as_raw()),
        ))?;
        Ok(())
    }

    /// Retrieves the current reverb environment for the specified reverb
    /// instance.
    pub fn get_reverb_properties(&self, instance: i32) -> Result<ReverbProperties> {
        let mut properties = ReverbProperties::default();
        ffi!(FMOD_System_GetReverbProperties(
            self.as_raw(),
            instance,
            properties.as_raw_mut(),
        ))?;
        Ok(properties)
    }

    /// Connect the output of the specified ChannelGroup to an audio port on the
    /// output driver.
    ///
    /// Ports are additional outputs supported by some [OutputType] plugins and
    /// can include things like controller headsets or dedicated background
    /// music streams. See the Port Support section (where applicable) of each
    /// platform's getting started guide found in the [platform details] chapter.
    ///
    /// [platform details]: https://fmod.com/resources/documentation-api?version=2.02&page=platforms.html
    pub fn attach_channel_group_to_port(
        &self,
        port_type: PortType,
        port_index: PortIndex,
        group: &ChannelGroup,
        pass_thru: bool,
    ) -> Result {
        ffi!(FMOD_System_AttachChannelGroupToPort(
            self.as_raw(),
            port_type.into_raw(),
            port_index.into_raw(),
            group.as_raw(),
            pass_thru as _,
        ))?;
        Ok(())
    }

    /// Disconnect the output of the specified ChannelGroup from an audio port
    /// on the output driver.
    ///
    /// Removing a [ChannelGroup] from a port will reroute the audio back to the
    /// main mix.
    pub fn detach_channel_group_from_port(&self, channel_group: &ChannelGroup) -> Result {
        ffi!(FMOD_System_DetachChannelGroupFromPort(
            self.as_raw(),
            channel_group.as_raw(),
        ))?;
        Ok(())
    }
}

fmod_struct! {
    /// Structure defining a reverb environment.
    ///
    /// The generic reverb properties are those used by [ReverbProperties::GENERIC].
    pub struct ReverbProperties = FMOD_REVERB_PROPERTIES {
        /// Reverberation decay time.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>1500</dd>
        /// <dt>Range</dt><dd>[0, 20000]</dd>
        /// </dl>
        #[default(1500.0)]
        pub decay_time: f32,
        /// Initial reflection delay time.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>7</dd>
        /// <dt>Range</dt><dd>[0, 300]</dd>
        /// </dl>
        #[default(7.0)]
        pub early_delay: f32,
        /// Late reverberation delay time relative to initial reflection.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>11</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(11.0)]
        pub late_delay: f32,
        /// Reference high frequency.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>5000</dd>
        /// <dt>Range</dt><dd>[20, 20000]</dd>
        /// </dl>
        #[default(5000.0)]
        pub hf_reference: f32,
        /// High-frequency to mid-frequency decay time ratio.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[10, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub hf_decay_ratio: f32,
        /// Value that controls the echo density in the late reverberation decay.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[10, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub diffusion: f32,
        /// Value that controls the modal density in the late reverberation decay.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>100</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(100.0)]
        pub density: f32,
        /// Reference low frequency
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>250</dd>
        /// <dt>Range</dt><dd>[20, 1000]</dd>
        /// </dl>
        #[default(250.0)]
        pub low_shelf_frequency: f32,
        /// Relative room effect level at low frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Decibels</dd>
        /// <dt>Default</dt><dd>0</dd>
        /// <dt>Range</dt><dd>[-36, 12]</dd>
        /// </dl>
        #[default(0.0)]
        pub low_shelf_gain: f32,
        /// Relative room effect level at high frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>200000</dd>
        /// <dt>Range</dt><dd>[0, 20000]</dd>
        /// </dl>
        #[default(200000.0)]
        pub high_cut: f32,
        /// Early reflections level relative to room effect.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub early_late_mix: f32,
        /// Room effect level at mid frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Decibels</dd>
        /// <dt>Default</dt><dd>-6</dd>
        /// <dt>Range</dt><dd>[-80, 20]</dd>
        /// </dl>
        #[default(-6.0)]
        pub wet_level: f32,
    }
}

fmod_flags! {
    /// Output type specific index for when there are multiple instances of a port type.
    pub struct PortIndex: FMOD_PORT_INDEX {
        /// Use when a port index is not required
        None = FMOD_PORT_INDEX_NONE as _,
        /// Use as a flag to indicate the intended controller is associated with a VR headset
        VrController = FMOD_PORT_INDEX_FLAG_VR_CONTROLLER as _,
    }
}

fmod_enum! {
    /// Port types available for routing audio.
    pub enum PortType: FMOD_PORT_TYPE {
        Music          = FMOD_PORT_TYPE_MUSIC,
        CopyrightMusic = FMOD_PORT_TYPE_COPYRIGHT_MUSIC,
        Voice          = FMOD_PORT_TYPE_VOICE,
        Controller     = FMOD_PORT_TYPE_CONTROLLER,
        Personal       = FMOD_PORT_TYPE_PERSONAL,
        Vibration      = FMOD_PORT_TYPE_VIBRATION,
        Aux            = FMOD_PORT_TYPE_AUX,
    }
}

/// The maximum number of global/physical reverb instances.
///
/// Each instance of a physical reverb is an instance of a [DspSfxReverb] dsp in
/// the mix graph. This is unrelated to the number of possible Reverb3D objects,
/// which is unlimited.
pub const REVERB_MAX_INSTANCES: usize = FMOD_REVERB_MAXINSTANCES as usize;

macro_rules! reverb {
    {
        $decay_time:expr,
        $early_delay:expr,
        $late_delay:expr,
        $hf_reference:expr,
        $hf_decay_ratio:expr,
        $diffusion:expr,
        $density:expr,
        $low_shelf_frequency:expr,
        $low_shelf_gain:expr,
        $high_cut:expr,
        $early_late_mix:expr,
        $wet_level:expr $(,)?
    } => {
        ReverbProperties {
            decay_time: $decay_time,
            early_delay: $early_delay,
            late_delay: $late_delay,
            hf_reference: $hf_reference,
            hf_decay_ratio: $hf_decay_ratio,
            diffusion: $diffusion,
            density: $density,
            low_shelf_frequency: $low_shelf_frequency,
            low_shelf_gain: $low_shelf_gain,
            high_cut: $high_cut,
            early_late_mix: $early_late_mix,
            wet_level: $wet_level,
        }
    };
}

#[rustfmt::skip]
/// Predefined reverb configurations.
/// 
/// To simplify usage, and avoid manually selecting reverb parameters,
/// a table of common presets is supplied for ease of use.
impl ReverbProperties {
    pub const OFF: Self =               reverb! {  1000.0,    7.0,  11.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0,    20.0,  96.0, -80.0 };
    pub const GENERIC: Self =           reverb! {  1500.0,    7.0,  11.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0, 14500.0,  96.0,  -8.0 };
    pub const PADDED_CELL: Self =       reverb! {   170.0,    1.0,   2.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   160.0,  84.0,  -7.8 };
    pub const ROOM: Self =              reverb! {   400.0,    2.0,   3.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0,  6050.0,  88.0,  -9.4 };
    pub const BATHROOM: Self =          reverb! {  1500.0,    7.0,  11.0, 5000.0,  54.0, 100.0,  60.0, 250.0, 0.0,  2900.0,  83.0,   0.5 };
    pub const LIVING_ROOM: Self =       reverb! {   500.0,    3.0,   4.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   160.0,  58.0, -19.0 };
    pub const STONE_ROOM: Self =        reverb! {  2300.0,   12.0,  17.0, 5000.0,  64.0, 100.0, 100.0, 250.0, 0.0,  7800.0,  71.0,  -8.5 };
    pub const AUDITORIUM: Self =        reverb! {  4300.0,   20.0,  30.0, 5000.0,  59.0, 100.0, 100.0, 250.0, 0.0,  5850.0,  64.0, -11.7 };
    pub const CONCERT_HALL: Self =      reverb! {  3900.0,   20.0,  29.0, 5000.0,  70.0, 100.0, 100.0, 250.0, 0.0,  5650.0,  80.0,  -9.8 };
    pub const CAVE: Self =              reverb! {  2900.0,   15.0,  22.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0, 20000.0,  59.0, -11.3 };
    pub const ARENA: Self =             reverb! {  7200.0,   20.0,  30.0, 5000.0,  33.0, 100.0, 100.0, 250.0, 0.0,  4500.0,  80.0,  -9.6 };
    pub const HANGAR: Self =            reverb! { 10000.0,   20.0,  30.0, 5000.0,  23.0, 100.0, 100.0, 250.0, 0.0,  3400.0,  72.0,  -7.4 };
    pub const CARPETED_HALLWAY: Self =  reverb! {   300.0,    2.0,  30.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   500.0,  56.0, -24.0 };
    pub const HALLWAY: Self =           reverb! {  1500.0,    7.0,  11.0, 5000.0,  59.0, 100.0, 100.0, 250.0, 0.0,  7800.0,  87.0,  -5.5 };
    pub const STONE_CORRIDOR: Self =    reverb! {   270.0,   13.0,  20.0, 5000.0,  79.0, 100.0, 100.0, 250.0, 0.0,  9000.0,  86.0,  -6.0 };
    pub const ALLEY: Self =             reverb! {  1500.0,    7.0,  11.0, 5000.0,  86.0, 100.0, 100.0, 250.0, 0.0,  8300.0,  80.0,  -9.8 };
    pub const FOREST: Self =            reverb! {  1500.0,  162.0,  88.0, 5000.0,  54.0,  79.0, 100.0, 250.0, 0.0,   760.0,  94.0, -12.3 };
    pub const CITY: Self =              reverb! {  1500.0,    7.0,  11.0, 5000.0,  67.0,  50.0, 100.0, 250.0, 0.0,  4050.0,  66.0, -26.0 };
    pub const MOUNTAINS: Self =         reverb! {  1500.0,  300.0, 100.0, 5000.0,  21.0,  27.0, 100.0, 250.0, 0.0,  1220.0,  82.0, -24.0 };
    pub const QUARRY: Self =            reverb! {  1500.0,   61.0,  25.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0,  3400.0, 100.0,  -5.0 };
    pub const PLAIN: Self =             reverb! {  1500.0,  179.0, 100.0, 5000.0,  50.0,  21.0, 100.0, 250.0, 0.0,  1670.0,  65.0, -28.0 };
    pub const PARKING_LOT: Self =       reverb! {  1700.0,    8.0,  12.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0, 20000.0,  56.0, -19.5 };
    pub const SEWER_PIPE: Self =        reverb! {  2800.0,   14.0,  21.0, 5000.0,  14.0,  80.0,  60.0, 250.0, 0.0,  3400.0,  66.0,   1.2 };
    pub const UNDERWATER: Self =        reverb! {  1500.0,    7.0,  11.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   500.0,  92.0,   7.0 };
}

/// Position, velocity, and orientation of a 3D sound listener.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ListenerAttributes3d {
    /// Position in 3D space used for panning and attenuation.
    pub pos: Vector,
    /// Velocity in 3D space used for doppler.
    pub vel: Vector,
    /// Forwards orientation.
    pub forward: Vector,
    /// Upwards orientation.
    pub up: Vector,
}
