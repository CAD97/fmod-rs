use {
    fmod::{raw::*, *},
    smart_default::SmartDefault,
    std::{
        ffi::c_void,
        ops::{Bound, Range, RangeBounds},
        ptr,
    },
};

opaque! {
    /// The shared APIs between [`Channel`] and [`ChannelGroup`].
    weak class ChannelControl = FMOD_CHANNELCONTROL, FMOD_ChannelControl_*;
}

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C functions that take FMOD_CHANNELCONTROL and cast to call into the C++ API.

/// # Playback.
impl ChannelControl {
    /// Retrieves the playing state.
    ///
    /// A Channel is considered playing after [`System::play_sound`] or
    /// [`System::play_dsp`], even if it is paused.
    ///
    /// A [`ChannelGroup`] is considered playing if it has any playing Channels.
    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        ffi!(FMOD_Channel_IsPlaying(self.as_raw() as _, &mut isplaying))?;
        Ok(isplaying != 0)
    }

    /// Stops the Channel (or all Channels in nested ChannelGroups) from playing.
    ///
    /// This will free up internal resources for reuse by the virtual voice system.
    ///
    /// [`Channel`]s are stopped automatically when their playback position
    /// reaches the length of the [`Sound`] being played. This is not the case
    /// however if the [`Channel`] is playing a DSP or the [`Sound`] is looping,
    /// in which case the [`Channel`] will continue playing until stop is called.
    /// Once stopped, the [`Channel`] handle will become invalid and can be
    /// discarded and any API calls made with it will return
    /// [`Error::InvalidHandle`].
    pub fn stop(&self) -> Result {
        ffi!(FMOD_Channel_Stop(self.as_raw() as _))?;
        Ok(())
    }

    /// Sets the paused state.
    ///
    /// Pause halts playback which effectively freezes [`Channel::get_position`]
    /// and [`ChannelControl::get_dsp_clock`] values.
    ///
    /// An individual pause state is kept for each object, pausing a parent
    /// [`ChannelGroup`] will effectively pause this object however when queried
    /// the individual pause state is returned.
    pub fn set_paused(&self, paused: bool) -> Result {
        let paused = paused as i32;
        ffi!(FMOD_Channel_SetPaused(self.as_raw() as _, paused))?;
        Ok(())
    }

    /// Retrieves the paused state.
    ///
    /// An individual pause state is kept for each object, a parent
    /// [`ChannelGroup`] being paused will effectively pause this object however
    /// when queried the individual pause state is returned.
    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        ffi!(FMOD_Channel_GetPaused(self.as_raw() as _, &mut paused))?;
        Ok(paused != 0)
    }

    /// Sets the playback mode that controls how this object behaves.
    ///
    /// Modes supported:
    ///
    /// - [`Mode::LoopOff`]
    /// - [`Mode::LoopNormal`]
    /// - [`Mode::LoopBidi`]
    /// - [`Mode::D2`]
    /// - [`Mode::D3`]
    /// - [`Mode::HeadRelative3d`]
    /// - [`Mode::WorldRelative3d`]
    /// - [`Mode::InverseRolloff3d`]
    /// - [`Mode::LinearRolloff3d`]
    /// - [`Mode::LinearSquareRolloff3d`]
    /// - [`Mode::InverseTaperedRolloff3d`]
    /// - [`Mode::CustomRolloff3d`]
    /// - [`Mode::IgnoreGeometry3d`]
    /// - [`Mode::VirtualPlayFromStart`]
    ///
    /// When changing the loop mode, sounds created with
    /// [`System::create_stream`] or [`Mode::CreateStream`] may have already
    /// been pre-buffered and executed their loop logic ahead of time before
    /// this call was even made. This is dependent on the size of the sound
    /// versus the size of the stream decode buffer (see [`CreateSoundEx`]).
    /// If this happens, you may need to reflush the stream buffer by calling
    /// [`Channel::set_position`]. Note this will usually only happen if you
    /// have sounds or loop points that are smaller than the stream decode
    /// buffer size.
    ///
    /// When changing the loop mode of sounds created with with
    /// [`System::create_sound`] or [`Mode::CreateSample`], if the sound was set
    /// up as [`Mode::LoopOff`], then set to [`Mode::LoopNormal`] with this
    /// function, the sound may click when playing the end of the sound.
    /// This is because the sound needs to be prepared for looping using
    /// [`Sound::set_mode`], by modifying the content of the PCM data
    /// (i.e. data past the end of the actual sample data) to allow the
    /// interpolators to read ahead without clicking. If you use
    /// [`ChannelControl::set_mode`] it will not do this (because different
    /// Channels may have different loop modes for the same sound) and may click
    /// if you try to set it to looping on an unprepared sound. If you want to
    /// change the loop mode at runtime it may be better to load the sound as
    /// looping first (or use [`Sound::set_mode`]), to let it prepare the data
    /// as if it was looping so that it does not click whenever
    /// [`ChannelControl::set_mode`] is used to turn looping on.
    ///
    /// If [`Mode::IgnoreGeometry3d`] or [`Mode::VirtualPlayFromStart`] is not
    /// specified, the flag will be cleared if it was specified previously.
    pub fn set_mode(&self, mode: Mode) -> Result {
        ffi!(FMOD_Channel_SetMode(self.as_raw() as _, mode.into_raw()))?;
        Ok(())
    }

    /// Retrieves the playback mode that controls how this object behaves.
    pub fn get_mode(&self) -> Result<Mode> {
        let mut mode = 0;
        ffi!(FMOD_Channel_GetMode(self.as_raw() as _, &mut mode))?;
        Ok(Mode::from_raw(mode))
    }

    /// Sets the relative pitch / playback rate.
    ///
    /// Scales playback frequency of [`Channel`] object or if issued on a
    /// [`ChannelGroup`] it scales the frequencies of all [`Channels`]
    /// contained in the [`ChannelGroup`].
    ///
    /// A pitch value of 0.5 represents half pitch (one octave down),
    /// 1.0 represents unmodified pitch, and
    /// 2.0 represents double pitch (one octave up).
    ///
    /// An individual pitch value is kept for each object, changing the pitch of
    /// a parent [`ChannelGroup`] will effectively alter the pitch of this
    /// object however when queried the individual pitch value is returned.
    pub fn set_pitch(&self, pitch: f32) -> Result {
        ffi!(FMOD_Channel_SetPitch(self.as_raw() as _, pitch))?;
        Ok(())
    }

    /// Retrieves the relative pitch / playback rate.
    ///
    /// A pitch value of 0.5 represents half pitch (one octave down),
    /// 1.0 represents unmodified pitch, and
    /// 2.0 represents double pitch (one octave up).
    ///
    /// An individual pitch value is kept for each object, a parent
    /// [`ChannelGroup`] pitch will effectively scale the pitch of this object
    /// however when queried the individual pitch value is returned.
    pub fn get_pitch(&self) -> Result<f32> {
        let mut pitch = 0.0;
        ffi!(FMOD_Channel_GetPitch(self.as_raw() as _, &mut pitch))?;
        Ok(pitch)
    }
}

/// # Volume levels.
impl ChannelControl {
    /// Retrieves an estimation of the output volume.
    ///
    /// Estimated volume is calculated based on 3D spatialization, occlusion,
    /// API volume levels and DSPs used.
    ///
    /// While this does not represent the actual waveform, [`Channel`]s playing
    /// FSB files will take into consideration the overall peak level of the
    /// file (if available).
    ///
    /// This value is used to determine which [`Channel`]s should be audible
    /// and which [`Channel`]s to virtualize when resources are limited.
    ///
    /// See the [Virtual Voice System][audibility-calculation] white paper for
    /// more details about how audibility is calculated.
    ///
    /// [audibility-calculation]: https://fmod.com/docs/2.02/api/white-papers-virtual-voices.html#audibility-calculation
    pub fn get_audibility(&self) -> Result<f32> {
        let mut audibility = 0.0;
        ffi!(FMOD_Channel_GetAudibility(
            self.as_raw() as _,
            &mut audibility
        ))?;
        Ok(audibility)
    }

    /// Sets the volume level.
    ///
    /// To define the volume per `Sound` use [`Sound::set_defaults`].
    ///
    /// Setting volume at a level higher than 1 can lead to distortion/clipping.
    pub fn set_volume(&self, volume: f32) -> Result {
        ffi!(FMOD_Channel_SetVolume(self.as_raw() as _, volume))?;
        Ok(())
    }

    /// Retrieves the volume level.
    ///
    /// Volume changes when not paused will be ramped to the target value to
    /// avoid a pop sound, this function allows that setting to be overridden
    /// and volume changes to be applied immediately.
    pub fn get_volume(&self) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_Channel_GetVolume(self.as_raw() as _, &mut volume))?;
        Ok(volume)
    }

    /// Sets whether volume changes are ramped or instantaneous.
    pub fn set_volume_ramp(&self, ramp: bool) -> Result {
        let ramp = ramp as i32;
        ffi!(FMOD_Channel_SetVolumeRamp(self.as_raw() as _, ramp))?;
        Ok(())
    }

    /// Retrieves whether volume changes are ramped or instantaneous.
    pub fn get_volume_ramp(&self) -> Result<bool> {
        let mut ramp = 0;
        ffi!(FMOD_Channel_GetVolumeRamp(self.as_raw() as _, &mut ramp))?;
        Ok(ramp != 0)
    }

    /// Sets the mute state.
    ///
    /// Mute is an additional control for volume, the effect of which is
    /// equivalent to setting the volume to zero.
    ///
    /// An individual mute state is kept for each object, muting a parent
    /// [`ChannelGroup`] will effectively mute this object however when queried
    /// the individual mute state is returned.
    /// [`ChannelControl::get_audibility`] can be used to calculate overall
    /// audibility for a [`Channel`] or [`ChannelGroup`].
    pub fn set_mute(&self, mute: bool) -> Result {
        ffi!(FMOD_Channel_SetMute(
            self.as_raw() as _,
            if mute { 1 } else { 0 },
        ))?;
        Ok(())
    }

    /// Retrieves the mute state.
    ///
    /// An individual mute state is kept for each object, a parent
    /// [`ChannelGroup`] being muted will effectively mute this object however
    /// when queried the individual mute state is returned.
    /// [`ChannelControl::get_audibility`] can be used to calculate overall
    /// audibility for a [`Channel`] or [`ChannelGroup`].
    pub fn get_mute(&self) -> Result<bool> {
        let mut mute = 0;
        ffi!(FMOD_Channel_GetMute(self.as_raw() as _, &mut mute))?;
        Ok(mute != 0)
    }
}

/// # Spatialization.
impl ChannelControl {
    /// Sets the 3D position and velocity used to apply panning,
    /// attenuation and doppler.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// Vectors must be provided in the correct [handedness].
    ///
    /// [handedness]: https://fmod.com/docs/2.02/api/glossary.html#handedness
    ///
    /// For a stereo 3D sound, you can set the spread of the left/right parts
    /// in speaker space by using [`ChannelControl::set_3d_spread`].
    pub fn set_3d_attributes(&self, pos: &Vector, vel: &Vector) -> Result {
        ffi!(FMOD_Channel_Set3DAttributes(
            self.as_raw() as _,
            pos.as_raw(),
            vel.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the 3D position and velocity used to apply panning,
    /// attenuation and doppler.
    pub fn get_3d_attributes(&self) -> Result<(Vector, Vector)> {
        let mut pos = Vector::default();
        let mut vel = Vector::default();
        ffi!(FMOD_Channel_Get3DAttributes(
            self.as_raw() as _,
            pos.as_raw_mut(),
            vel.as_raw_mut(),
        ))?;
        Ok((pos, vel))
    }

    /// Sets the orientation of a 3D cone shape, used for simulated occlusion.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// This function has no effect unless
    /// [`ChannelControl::set_3d_cone_settings`] has been used to change the
    /// cone inside/outside angles from the default.
    ///
    /// Vectors must be provided in the correct [handedness].
    ///
    /// [handedness]: https://fmod.com/docs/2.02/api/glossary.html#handedness
    pub fn set_3d_cone_orientation(&self, orientation: &Vector) -> Result {
        ffi!(FMOD_Channel_Set3DConeOrientation(
            self.as_raw() as _,
            orientation.as_raw() as *const _ as _,
        ))?;
        Ok(())
    }

    /// Retrieves the orientation of a 3D cone shape,
    /// used for simulated occlusion.
    pub fn get_3d_cone_orientation(&self) -> Result<Vector> {
        let mut orientation = Vector::default();
        ffi!(FMOD_Channel_Get3DConeOrientation(
            self.as_raw() as _,
            orientation.as_raw_mut(),
        ))?;
        Ok(orientation)
    }

    /// Sets the angles and attenuation levels of a 3D cone shape,
    /// for simulated occlusion which is based on direction.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// When [`ChannelControl::set_3d_cone_orientation`] is used and a 3D 'cone'
    /// is set up, attenuation will automatically occur for a sound based on the
    /// relative angle of the direction the cone is facing, vs the angle between
    /// the sound and the listener.
    ///
    /// - If the relative angle is within the `inside_angle`, the sound will not
    ///   have any attenuation applied.
    /// - If the relative angle is between the `inside_angle` and
    ///   `outside_angle`, linear volume attenuation (between 1 and
    ///   `outside_volume`) is applied between the two angles until it reaches
    ///   the `outside_angle`.
    /// - If the relative angle is outside of the `outside_angle` the volume
    ///   does not attenuate any further.
    pub fn set_3d_cone_settings(&self, settings: Cone3dSettings) -> Result {
        ffi!(FMOD_Channel_Set3DConeSettings(
            self.as_raw() as _,
            settings.inside_angle,
            settings.outside_angle,
            settings.outside_volume,
        ))?;
        Ok(())
    }

    /// Retrieves the angles and attenuation levels of a 3D cone shape,
    /// for simulated occlusion which is based on direction.
    ///
    /// When [`ChannelControl::set_3d_cone_orientation`] is used and a 3D 'cone'
    /// is set up, attenuation will automatically occur for a sound based on the
    /// relative angle of the direction the cone is facing, vs the angle between
    /// the sound and the listener.
    ///
    /// - If the relative angle is within the `inside_angle`, the sound will not
    ///   have any attenuation applied.
    /// - If the relative angle is between the `inside_angle` and
    ///   `outside_angle`, linear volume attenuation (between 1 and
    ///   `outside_volume`) is applied between the two angles until it reaches
    ///   the `outside_angle`.
    /// - If the relative angle is outside of the `outside_angle` the volume
    ///   does not attenuate any further.
    pub fn get_3d_cone_settings(&self) -> Result<Cone3dSettings> {
        let mut cone = Cone3dSettings::default();
        ffi!(FMOD_Channel_Get3DConeSettings(
            self.as_raw() as _,
            &mut cone.inside_angle,
            &mut cone.outside_angle,
            &mut cone.outside_volume,
        ))?;
        Ok(cone)
    }

    // TODO: needs figuring out lifetimes
    // set_3d_custom_rolloff
    // get_3d_custom_rolloff

    /// Sets an override value for the 3D distance filter.
    ///
    /// If distance filtering is enabled, by default the 3D engine will
    /// automatically attenuate frequencies using a lowpass and a highpass
    /// filter, based on 3D distance. This function allows the distance filter
    /// effect to be set manually, or to be set back to 'automatic' mode.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// The [`System`] must be initialized with
    /// [`InitFlags::ChannelDistanceFilter`] for this feature to work.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn set_3d_distance_filter(&self, filter: Option<DistanceFilter3d>) -> Result {
        ffi!(FMOD_Channel_Set3DDistanceFilter(
            self.as_raw() as _,
            filter.is_some() as _,
            filter.map(|filter| filter.custom_level).unwrap_or(1.0),
            filter.map(|filter| filter.center_freq).unwrap_or(0.0),
        ))?;
        Ok(())
    }

    /// Retrieves the override values for the 3D distance filter.
    pub fn get_3d_distance_filter(&self) -> Result<Option<DistanceFilter3d>> {
        let mut custom = 0;
        let mut custom_level = 0.0;
        let mut center_freq = 0.0;
        ffi!(FMOD_Channel_Get3DDistanceFilter(
            self.as_raw() as _,
            &mut custom,
            &mut custom_level,
            &mut center_freq,
        ))?;
        if custom != 0 {
            Ok(Some(DistanceFilter3d {
                custom_level,
                center_freq,
            }))
        } else {
            Ok(None)
        }
    }

    /// Sets the amount by which doppler is scaled.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// The doppler effect will disabled if [`System::set_3d_num_listeners`]
    /// is given a value greater than 1.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn set_3d_doppler_level(&self, level: f32) -> Result {
        ffi!(FMOD_Channel_Set3DDopplerLevel(self.as_raw() as _, level))?;
        Ok(())
    }

    /// Retrieves the amount by which doppler is scaled.
    pub fn get_3d_doppler_level(&self) -> Result<f32> {
        let mut level = 0.0;
        ffi!(FMOD_Channel_Get3DDopplerLevel(
            self.as_raw() as _,
            &mut level,
        ))?;
        Ok(level)
    }

    /// Sets the blend between 3D panning and 2D panning.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// 2D functions include:
    ///
    /// - [`ChannelControl::set_pan`]
    /// - [`ChannelControl::set_mix_levels_output`]
    /// - [`ChannelControl::set_mix_levels_input`]
    /// - [`ChannelControl::set_mix_matrix`]
    ///
    /// 3D functions include:
    ///
    /// - [`ChannelControl::set_3d_attributes`]
    /// - [`ChannelControl::set_3d_cone_orientation`]
    /// - [`ChannelControl::set_3d_custom_rolloff`]
    pub fn set_3d_level(&self, level: f32) -> Result {
        ffi!(FMOD_Channel_Set3DLevel(self.as_raw() as _, level))?;
        Ok(())
    }

    /// Retrieves the blend between 3D panning and 2D panning.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// 2D functions include:
    ///
    /// - [`ChannelControl::set_pan`]
    /// - [`ChannelControl::set_mix_levels_output`]
    /// - [`ChannelControl::set_mix_levels_input`]
    /// - [`ChannelControl::set_mix_matrix`]
    ///
    /// 3D functions include:
    ///
    /// - [`ChannelControl::set_3d_attributes`]
    /// - [`ChannelControl::set_3d_cone_orientation`]
    /// - [`ChannelControl::set_3d_custom_rolloff`]
    pub fn get_3d_level(&self) -> Result<f32> {
        let mut level = 0.0;
        ffi!(FMOD_Channel_Get3DLevel(self.as_raw() as _, &mut level))?;
        Ok(level)
    }

    /// Sets the minimum and maximum distances used to calculate the
    /// 3D roll-off attenuation.
    ///
    /// When the listener is within the minimum distance of the sound source
    /// the 3D volume will be at its maximum. As the listener moves from the
    /// minimum distance to the maximum distance the sound will attenuate
    /// following the roll-off curve set. When outside the maximum distance
    /// the sound will no longer attenuate.
    ///
    /// Attenuation in 3D space is controlled by the roll-off mode, these are
    /// [`Mode::InverseRolloff3d`], [`Mode::LinearRolloff3d`],
    /// [`Mode::LinearSquareRolloff3d`], [`Mode::InverseTaperedRolloff3d`],
    /// [`Mode::CustomRolloff3d`].
    ///
    /// Minimum distance is useful to give the impression that the sound is
    /// loud or soft in 3D space. A sound with a small 3D minimum distance in
    /// a typical (non custom) roll-off mode will make the sound appear small,
    /// and the sound will attenuate quickly. A sound with a large minimum
    /// distance will make the sound appear larger.
    ///
    /// The [`Mode::D3`] flag must be set on this object otherwise
    /// [`Error::Needs3d`] is returned.
    ///
    /// To define the min and max distance per [`Sound`] instead of [`Channel`]
    /// or [`ChannelGroup`] use [`Sound::set_3d_min_max_distance`].
    ///
    /// If [`Mode::CustomRolloff3d`] has been set on this object
    /// these values are stored, but ignored in 3D processing.
    pub fn set_3d_min_max_distance(&self, distance: impl RangeBounds<f32>) -> Result {
        let min_distance = match distance.start_bound() {
            Bound::Included(&min_distance) => min_distance,
            Bound::Excluded(&min_distance) => min_distance,
            Bound::Unbounded => 0.0,
        };
        let max_distance = match distance.end_bound() {
            Bound::Included(&max_distance) => max_distance,
            Bound::Excluded(&max_distance) => max_distance,
            Bound::Unbounded => f32::INFINITY,
        };
        ffi!(FMOD_Channel_Set3DMinMaxDistance(
            self.as_raw() as _,
            min_distance,
            max_distance,
        ))?;
        Ok(())
    }

    /// Retrieves the minimum and maximum distances used to calculate the 3D roll-off attenuation.
    pub fn get_3d_min_max_distance(&self) -> Result<Range<f32>> {
        let mut distance = 0.0..0.0;
        ffi!(FMOD_Channel_Get3DMinMaxDistance(
            self.as_raw() as _,
            &mut distance.start,
            &mut distance.end,
        ))?;
        Ok(distance)
    }

    /// Sets the 3D attenuation factors for the direct and reverb paths.
    ///
    /// There is a reverb path/send when
    /// [`ChannelControl::set_reverb_properties`] has been used,
    /// [`occlusion.reverb`] controls its attenuation.
    ///
    /// If the [`System`] has been initialized with
    /// [`InitFlags::ChannelDistanceFilter`] or [`InitFlags::ChannelLowpass`]
    /// the `occlusion.direct` is applied as frequency filtering rather than
    /// volume attenuation.
    pub fn set_3d_occlusion(&self, occlusion: Occlusion) -> Result {
        ffi!(FMOD_Channel_Set3DOcclusion(
            self.as_raw() as _,
            occlusion.direct,
            occlusion.reverb,
        ))?;
        Ok(())
    }

    /// Retrieves the 3D attenuation factors for the direct and reverb paths.
    pub fn get_3d_occlusion(&self) -> Result<Occlusion> {
        let mut occlusion = Occlusion::default();
        ffi!(FMOD_Channel_Get3DOcclusion(
            self.as_raw() as _,
            &mut occlusion.direct,
            &mut occlusion.reverb,
        ))?;
        Ok(occlusion)
    }

    /// Sets the spread of a 3D sound in speaker space.
    pub fn set_3d_spread(&self, angle: f32) -> Result {
        ffi!(FMOD_Channel_Set3DSpread(self.as_raw() as _, angle))?;
        Ok(())
    }
}

/// Override values for the 3D distance filter.
///
/// If distance filtering is enabled, by default the 3D engine will
/// automatically attenuate frequencies using a lowpass and a highpass filter,
/// based on 3D distance.
#[derive(Debug, Clone, Copy, SmartDefault, PartialEq)]
pub struct DistanceFilter3d {
    /// Attenuation factor where 1 represents no attenuation and 0 represents
    /// complete attenuation.
    #[default(1.0)]
    pub custom_level: f32,
    /// Center frequency of the band-pass filter used to simulate distance
    /// attenuation, 0 for default.
    #[default(0.0)]
    pub center_freq: f32,
}

/// # Panning and level adjustment.
impl ChannelControl {
    /// Sets the left/right pan level.
    ///
    /// This is a convenience function to avoid passing a matrix, it will
    /// overwrite values set via [`ChannelControl::set_mix_levels_input`],
    /// [`ChannelControl::set_mix_levels_output`] and
    /// [`ChannelControl::set_mix_matrix`].
    ///
    /// Mono inputs are panned from left to right using constant power panning
    /// (non linear fade). Stereo and greater inputs will isolate the front left
    /// and right input channels and fade them up and down based on the pan
    /// value (silencing other channels). The output channel count will always
    /// match the [`System`] speaker mode set via
    /// [`System::set_software_format`].
    ///
    /// If the System is initialized with [`SpeakerMode::Raw`] calling this
    /// function will produce silence.
    pub fn set_pan(&self, pan: f32) -> Result {
        ffi!(FMOD_Channel_SetPan(self.as_raw() as _, pan))?;
        Ok(())
    }

    /// Sets the incoming volume level for each channel of a multi-channel signal.
    ///
    /// This is a convenience function to avoid passing a matrix,
    /// it will overwrite values set via [ChannelControl::set_pan],
    /// [ChannelControl::set_mix_levels_output], and
    /// [ChannelControl::set_mix_matrix].
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn set_mix_levels_input(&self, levels: &[f32]) -> Result {
        ffi!(FMOD_Channel_SetMixLevelsInput(
            self.as_raw() as _,
            levels.as_ptr() as _,
            levels.len() as _,
        ))?;
        Ok(())
    }

    /// Sets the outgoing volume levels for each speaker.
    ///
    /// Specify the level for a given output speaker, if the channel count of
    /// the input and output do not match, channels will be up/down mixed as
    /// appropriate to approximate the given speaker values. For example stereo
    /// input with 5.1 output will use the `center` parameter to distribute
    /// signal to the center speaker from the front left and front right
    /// channels.
    ///
    /// This is a convenience function to avoid passing a matrix,
    /// it will overwrite values set via [ChannelControl::set_pan],
    /// [ChannelControl::set_mix_levels_input], and
    /// [ChannelControl::set_mix_matrix].
    ///
    /// The output channel count will always match the System speaker mode set
    /// via [System::set_software_format].
    ///
    /// If the System is initialized with [InitFlags::SpeakerModeRaw] calling
    /// this function will produce silence.
    #[allow(clippy::too_many_arguments)] // it's on FMOD, not me
    pub fn set_mix_levels_output(
        &self,
        front_left: f32,
        front_right: f32,
        center: f32,
        lfe: f32,
        surround_left: f32,
        surround_right: f32,
        back_left: f32,
        back_right: f32,
    ) -> Result {
        ffi!(FMOD_Channel_SetMixLevelsOutput(
            self.as_raw() as _,
            front_left,
            front_right,
            center,
            lfe,
            surround_left,
            surround_right,
            back_left,
            back_right,
        ))?;
        Ok(())
    }

    // TODO: needs figuring out how to handle matrices
    // set_mix_matrix
    // get_mix_matrix
}

/// # Filtering.
impl ChannelControl {
    /// Sets the wet / send level for a particular reverb instance.
    ///
    /// [`Channel`]s are automatically connected to all existing reverb
    /// instances due to the default wet level of 1. [`ChannelGroup`]s however
    /// will not send to any reverb by default requiring an explicit call to
    /// this function.
    ///
    /// [`ChannelGroup`] reverb is optimal for the case where you want to send 1
    /// mixed signal to the reverb, rather than a lot of individual [`Channel`]
    /// reverb sends. It is advisable to do this to reduce CPU if you have many
    /// [`Channel`]s inside a [`ChannelGroup`].
    ///
    /// When setting a wet level for a [`ChannelGroup`], any [`Channel`]s under
    /// that [`ChannelGroup`] will still have their existing sends to the
    /// reverb. To avoid this doubling up you should explicitly set the
    /// [`Channel`] wet levels to 0.
    pub fn set_reverb_properties(&self, instance: i32, wet: f32) -> Result {
        ffi!(FMOD_Channel_SetReverbProperties(
            self.as_raw() as _,
            instance,
            wet,
        ))?;
        Ok(())
    }

    /// Retrieves the wet / send level for a particular reverb instance.
    pub fn get_reverb_properties(&self, instance: i32) -> Result<f32> {
        let mut wet = 0.0;
        ffi!(FMOD_Channel_GetReverbProperties(
            self.as_raw() as _,
            instance,
            &mut wet,
        ))?;
        Ok(wet)
    }

    /// Sets the gain of the dry signal when built in lowpass / distance
    /// filtering is applied.
    ///
    /// Requires the built in lowpass to be created with
    /// [`InitFlags::ChannelLowpass`] or [`InitFlags::ChannelDistanceFilter`].
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn set_low_pass_gain(&self, gain: f32) -> Result {
        ffi!(FMOD_Channel_SetLowPassGain(self.as_raw() as _, gain))?;
        Ok(())
    }

    /// Retrieves the gain of the dry signal when built in lowpass / distance
    /// filtering is applied.
    ///
    /// Requires the built in lowpass to be created with
    /// [`InitFlags::ChannelLowpass`] or [`InitFlags::ChannelDistanceFilter`].
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Currently only supported for Channel, not ChannelGroup.
    /// </span></div></div>
    pub fn get_low_pass_gain(&self) -> Result<f32> {
        let mut gain = 0.0;
        ffi!(FMOD_Channel_GetLowPassGain(self.as_raw() as _, &mut gain))?;
        Ok(gain)
    }
}

/// # DSP chain configuration
impl ChannelControl {
    /// Head of the DSP chain, equivalent of index 0.
    pub const DSP_HEAD: i32 = FMOD_CHANNELCONTROL_DSP_HEAD;
    /// Built in fader DSP.
    pub const DSP_FADER: i32 = FMOD_CHANNELCONTROL_DSP_FADER;
    /// Tail of the DSP chain, equivalent of the number of DSPs minus 1.
    pub const DSP_TAIL: i32 = FMOD_CHANNELCONTROL_DSP_TAIL;

    /// Adds a DSP unit to the specified index in the DSP chain.
    ///
    /// If `dsp` is already added to an existing object it will be removed and
    /// then added to this object.
    ///
    /// For detailed information on FMOD's DSP network, read the
    /// [DSP Architecture and Usage] white paper.
    ///
    /// [DSP Architecture and Usage]: https://fmod.com/docs/2.02/api/white-papers-dsp-architecture.html
    pub fn add_dsp(&self, index: i32, dsp: &Dsp) -> Result {
        ffi!(FMOD_Channel_AddDSP(self.as_raw() as _, index, dsp.as_raw()))?;
        Ok(())
    }

    /// Removes the specified DSP unit from the DSP chain.
    pub fn remove_dsp(&self, dsp: &Dsp) -> Result {
        ffi!(FMOD_Channel_RemoveDSP(self.as_raw() as _, dsp.as_raw()))?;
        Ok(())
    }

    /// Retrieves the number of DSP units in the DSP chain.
    pub fn get_num_dsps(&self) -> Result<i32> {
        let mut num_dsps = 0;
        ffi!(FMOD_Channel_GetNumDSPs(self.as_raw() as _, &mut num_dsps))?;
        Ok(num_dsps)
    }

    /// Sets the index in the DSP chain of the specified DSP.
    ///
    /// This will move a [`Dsp`] already in the [DSP chain] to a new offset.
    ///
    /// [DSP chain]: https://fmod.com/docs/2.02/api/glossary.html#dsp-chain
    pub fn set_dsp_index(&self, dsp: &Dsp, index: i32) -> Result {
        ffi!(FMOD_Channel_SetDSPIndex(
            self.as_raw() as _,
            dsp.as_raw(),
            index,
        ))?;
        Ok(())
    }

    /// Retrieves the index of a DSP inside the Channel or ChannelGroup's
    /// DSP chain.
    ///
    /// See [DSP chain].
    ///
    /// [DSP chain]: https://fmod.com/docs/2.02/api/glossary.html#dsp-chain
    pub fn get_dsp_index(&self, dsp: &Dsp) -> Result<i32> {
        let mut index = 0;
        ffi!(FMOD_Channel_GetDSPIndex(
            self.as_raw() as _,
            dsp.as_raw(),
            &mut index,
        ))?;
        Ok(index)
    }
}

/// # Sample accurate scheduling
impl ChannelControl {
    /// Retrieves the DSP clock value for the tail DSP node.
    ///
    /// To perform sample accurate scheduling in conjunction with
    /// [`ChannelControl::set_delay`] and [`ChannelControl::add_fade_point`]
    /// use [`ChannelControl::get_parent_dsp_clock`].
    pub fn get_dsp_clock(&self) -> Result<u64> {
        let mut dsp_clock = 0;
        ffi!(FMOD_Channel_GetDSPClock(
            self.as_raw() as _,
            &mut dsp_clock,
            ptr::null_mut()
        ))?;
        Ok(dsp_clock)
    }

    /// Retrieves the DSP clock value for the tail DSP node of the parent
    /// [`ChannelGroup`].
    pub fn get_parent_dsp_clock(&self) -> Result<u64> {
        let mut parent_clock = 0;
        ffi!(FMOD_Channel_GetDSPClock(
            self.as_raw() as _,
            ptr::null_mut(),
            &mut parent_clock,
        ))?;
        Ok(parent_clock)
    }

    /// Sets a sample accurate start (and/or stop) time relative to the parent
    /// ChannelGroup DSP clock.
    pub fn set_delay(&self, dsp_clock: impl RangeBounds<u64>, stop_channels: StopAction) -> Result {
        let dsp_clock_start = match dsp_clock.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let dsp_clock_end = match dsp_clock.end_bound() {
            Bound::Included(&end) => end,
            Bound::Excluded(&end) => end.saturating_sub(1),
            Bound::Unbounded => 0,
        };
        ffi!(FMOD_Channel_SetDelay(
            self.as_raw() as _,
            dsp_clock_start,
            dsp_clock_end,
            stop_channels as _,
        ))?;
        Ok(())
    }

    /// Retrieves a sample accurate start (and/or stop) time relative to the
    /// parent ChannelGroup DSP clock.
    pub fn get_delay(&self) -> Result<(Range<u64>, StopAction)> {
        let mut dsp_clock = 0..0;
        let mut stop_channels = 0;
        ffi!(FMOD_Channel_GetDelay(
            self.as_raw() as _,
            &mut dsp_clock.start,
            &mut dsp_clock.end,
            &mut stop_channels,
        ))?;
        Ok((dsp_clock, StopAction::from_raw(stop_channels)))
    }

    /// Adds a sample accurate fade point at a time relative to the parent
    /// ChannelGroup DSP clock.
    ///
    /// Fade points are scaled against other volume settings and in-between
    /// each fade point the volume will be linearly ramped.
    ///
    /// To perform sample accurate fading use
    /// [`ChannelControl::get_parent_dsp_clock`] to query the parent clock
    /// value. If a parent [`ChannelGroup`] changes its pitch, the fade points
    /// will still be correct as the parent clock rate is adjusted by that pitch.
    pub fn add_fade_point(&self, dsp_clock: u64, volume: f32) -> Result {
        ffi!(FMOD_Channel_AddFadePoint(
            self.as_raw() as _,
            dsp_clock,
            volume,
        ))?;
        Ok(())
    }

    /// Adds a volume ramp at the specified time in the future using fade points.
    ///
    /// This is a convenience function that creates a scheduled 64 sample fade
    /// point ramp from the current volume level to volume arriving at
    /// `dsp_clock` time.
    ///
    /// Can be use in conjunction with [`ChannelControl::set_delay`.
    ///
    /// All fade points after `dsp_clock` will be removed.
    pub fn set_fade_point_ramp(&self, dsp_clock: u64, volume: f32) -> Result {
        ffi!(FMOD_Channel_SetFadePointRamp(
            self.as_raw() as _,
            dsp_clock,
            volume,
        ))?;
        Ok(())
    }

    /// Removes all fade points in the specified clock range.
    pub fn remove_fade_points(&self, clock: impl RangeBounds<u64>) -> Result {
        let clock_start = match clock.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let clock_end = match clock.end_bound() {
            Bound::Included(&end) => end,
            Bound::Excluded(&end) => end.saturating_sub(1),
            Bound::Unbounded => u64::MAX,
        };
        ffi!(FMOD_Channel_RemoveFadePoints(
            self.as_raw() as _,
            clock_start,
            clock_end,
        ))?;
        Ok(())
    }

    // TODO: this could probably have a better interface
    /// Retrieves information about stored fade points.
    ///
    /// Passing `None` for both `dsp_clock` and `volume` will query the number
    /// of fade points stored. If either are `Some`, the return is the number
    /// of fadepoints retrieved, which is bounded by the size of the slice(s).
    pub fn get_fade_points(
        &self,
        clock: Option<&mut [u64]>,
        volume: Option<&mut [f32]>,
    ) -> Result<usize> {
        let mut num_points = u32::min(
            clock.as_deref().map(|s| s.len() as _).unwrap_or(u32::MAX),
            volume.as_deref().map(|s| s.len() as _).unwrap_or(u32::MAX),
        );
        let point_dsp_clock = clock.map(|s| s.as_mut_ptr()).unwrap_or(ptr::null_mut());
        let point_volume = volume.map(|s| s.as_mut_ptr()).unwrap_or(ptr::null_mut());
        ffi!(FMOD_Channel_GetFadePoints(
            self.as_raw() as _,
            &mut num_points,
            point_dsp_clock,
            point_volume,
        ))?;
        Ok(num_points as _)
    }
}

/// A scheduled action to stop playing sound.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
pub enum StopAction {
    #[default]
    /// Behaves like [ChannelControl::stop] has been called.
    Stop = true as _,
    /// Behaves like [ChannelControl::set_paused] has been called,
    /// a subsequent start allows it to resume.
    Pause = false as _,
}

impl StopAction {
    raw! {
        #[inline]
        pub fn from_raw(raw: FMOD_BOOL) -> Self {
            match raw {
                0 => StopAction::Pause,
                _ => StopAction::Stop,
            }
        }
    }
}

/// # General.
impl ChannelControl {
    /// Sets the callback for ChannelControl level notifications.
    pub fn set_callback<C: ChannelControlCallback>(&self) -> Result {
        ffi!(FMOD_Channel_SetCallback(
            self.as_raw() as _,
            Some(channel_control_callback::<C>),
        ))?;
        Ok(())
    }

    // TODO: needs figuring out type memory
    // set_user_data
    // get_user_data

    /// Retrieves the System that created this object.
    pub fn get_system_object(&self) -> Result<&System> {
        let mut system = ptr::null_mut();
        ffi!(FMOD_Channel_GetSystemObject(
            self.as_raw() as _,
            &mut system
        ))?;
        Ok(unsafe { System::from_raw(system) })
    }
}

pub trait ChannelControlCallback: ChannelCallback + ChannelGroupCallback {}
impl<C: ChannelCallback + ChannelGroupCallback> ChannelControlCallback for C {}

pub(crate) unsafe extern "system" fn channel_control_callback<C: ChannelControlCallback>(
    channelcontrol: *mut FMOD_CHANNELCONTROL,
    controltype: FMOD_CHANNELCONTROL_TYPE,
    callbacktype: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    commanddata1: *mut c_void,
    commanddata2: *mut c_void,
) -> FMOD_RESULT {
    let callback = match controltype {
        FMOD_CHANNELCONTROL_CHANNEL => channel_callback::<C>,
        FMOD_CHANNELCONTROL_CHANNELGROUP => channel_group_callback::<C>,
        _ => {
            whoops!(no_panic: "unknown channel control type: {:?}", controltype);
            return FMOD_ERR_INVALID_PARAM;
        },
    };
    callback(
        channelcontrol,
        controltype,
        callbacktype,
        commanddata1,
        commanddata2,
    )
}
