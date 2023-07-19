use {
    fmod::{raw::*, *},
    smart_default::SmartDefault,
    std::ops::{Bound, Range, RangeBounds},
};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

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

/// Angles and attenuation levels of a 3D cone shape,
/// for simulated occlusion which is based on direction.
#[derive(Debug, Clone, Copy, SmartDefault, PartialEq)]
pub struct Cone3dSettings {
    /// Inside cone angle. This is the angle spread within which the sound
    /// is unattenuated.
    /// <dl>
    /// <dt>Units</dt><dd>Degrees</dd>
    /// <dt>Range</dt><dd>[0, <code>outside_angle</code>]</dd>
    /// <dt>Default</dt><dd>360</dd>
    /// </dl>
    #[default(360.0)]
    pub inside_angle: f32,
    /// Outside cone angle. This is the angle spread outside of which the sound
    /// is attenuated to its `outside_volume`.
    /// <dl>
    /// <dt>Units</dt><dd>Degrees</dd>
    /// <dt>Range</dt><dd>[<code>inside_angle</code>, 360]</dd>
    /// <dt>Default</dt><dd>360</dd>
    /// </dl>
    #[default(360.0)]
    pub outside_angle: f32,
    /// Cone outside volume.
    /// <dl>
    /// <dt>Units</dt><dd>Linear</dd>
    /// <dt>Range</dt><dd>[0, 1]</dd>
    /// <dt>Default</dt><dd>1</dd>
    /// </dl>
    #[default(1.0)]
    pub outside_volume: f32,
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
