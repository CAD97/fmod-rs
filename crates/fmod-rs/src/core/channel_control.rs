use {
    fmod::{raw::*, *},
    std::{
        ffi::c_void,
        ops::{Bound, Range, RangeBounds},
        ptr,
    },
};

opaque! {
    /// The shared APIs between [Channel] and [ChannelGroup].
    weak class ChannelControl = FMOD_CHANNELCONTROL, FMOD_ChannelControl_*;
}

/// # Playback
impl ChannelControl {
    /// Retrieves the playing state.
    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        ffi!(FMOD_Channel_IsPlaying(self.as_raw() as _, &mut isplaying))?;
        Ok(isplaying != 0)
    }

    /// Stops the Channel (or all Channels in nested ChannelGroups) from playing.
    pub fn stop(&self) -> Result {
        ffi!(FMOD_Channel_Stop(self.as_raw() as _))?;
        Ok(())
    }

    /// Sets the paused state.
    pub fn set_paused(&self, paused: bool) -> Result {
        let paused = paused as i32;
        ffi!(FMOD_Channel_SetPaused(self.as_raw() as _, paused))?;
        Ok(())
    }

    /// Retrieves the paused state.
    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        ffi!(FMOD_Channel_GetPaused(self.as_raw() as _, &mut paused))?;
        Ok(paused != 0)
    }

    /// Sets the playback mode that controls how this object behaves.
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
    pub fn set_pitch(&self, pitch: f32) -> Result {
        ffi!(FMOD_Channel_SetPitch(self.as_raw() as _, pitch))?;
        Ok(())
    }

    /// Retrieves the relative pitch / playback rate.
    pub fn get_pitch(&self) -> Result<f32> {
        let mut pitch = 0.0;
        ffi!(FMOD_Channel_GetPitch(self.as_raw() as _, &mut pitch))?;
        Ok(pitch)
    }
}

/// # Volume levels
impl ChannelControl {
    /// Retrieves an estimation of the output volume.
    pub fn get_audibility(&self) -> Result<f32> {
        let mut audibility = 0.0;
        ffi!(FMOD_Channel_GetAudibility(
            self.as_raw() as _,
            &mut audibility
        ))?;
        Ok(audibility)
    }

    /// Sets the volume level.
    pub fn set_volume(&self, volume: f32) -> Result {
        ffi!(FMOD_Channel_SetVolume(self.as_raw() as _, volume))?;
        Ok(())
    }

    /// Retrieves the volume level.
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
    pub fn set_mute(&self, mute: bool) -> Result {
        ffi!(FMOD_Channel_SetMute(
            self.as_raw() as _,
            if mute { 1 } else { 0 },
        ))?;
        Ok(())
    }

    /// Retrieves the mute state.
    pub fn get_mute(&self) -> Result<bool> {
        let mut mute = 0;
        ffi!(FMOD_Channel_GetMute(self.as_raw() as _, &mut mute))?;
        Ok(mute != 0)
    }
}

/// # Spatialization
impl ChannelControl {
    /// Sets the 3D position and velocity used to apply panning, attenuation and doppler.
    pub fn set_3d_attributes(&self, pos: &Vector, vel: &Vector) -> Result {
        ffi!(FMOD_Channel_Set3DAttributes(
            self.as_raw() as _,
            pos.as_raw(),
            vel.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the 3D position and velocity used to apply panning, attenuation and doppler.
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
    pub fn set_3d_cone_orientation(&self, orientation: &Vector) -> Result {
        ffi!(FMOD_Channel_Set3DConeOrientation(
            self.as_raw() as _,
            orientation.as_raw() as *const _ as _,
        ))?;
        Ok(())
    }

    /// Retrieves the orientation of a 3D cone shape, used for simulated occlusion.
    pub fn get_3d_cone_orientation(&self) -> Result<Vector> {
        let mut orientation = Vector::default();
        ffi!(FMOD_Channel_Get3DConeOrientation(
            self.as_raw() as _,
            orientation.as_raw_mut(),
        ))?;
        Ok(orientation)
    }

    /// Sets the angles and attenuation levels of a 3D cone shape, for simulated occlusion which is based on direction.
    pub fn set_3d_cone_settings(
        &self,
        inside_cone_angle: f32,
        outside_cone_angle: f32,
        outside_volume: f32,
    ) -> Result {
        ffi!(FMOD_Channel_Set3DConeSettings(
            self.as_raw() as _,
            inside_cone_angle,
            outside_cone_angle,
            outside_volume,
        ))?;
        Ok(())
    }

    /// Retrieves the angles and attenuation levels of a 3D cone shape, for simulated occlusion which is based on direction.
    pub fn get_3d_cone_settings(&self) -> Result<(f32, f32, f32)> {
        let mut inside_cone_angle = 0.0;
        let mut outside_cone_angle = 0.0;
        let mut outside_volume = 0.0;
        ffi!(FMOD_Channel_Get3DConeSettings(
            self.as_raw() as _,
            &mut inside_cone_angle,
            &mut outside_cone_angle,
            &mut outside_volume,
        ))?;
        Ok((inside_cone_angle, outside_cone_angle, outside_volume))
    }

    // TODO: needs figuring out lifetimes
    // set_3d_custom_rolloff
    // get_3d_custom_rolloff

    // TODO: needs figuring out option semantics:
    // set_3d_distance_filter
    // get_3d_distance_filter

    /// Sets the amount by which doppler is scaled.
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
    pub fn set_3d_level(&self, level: f32) -> Result {
        ffi!(FMOD_Channel_Set3DLevel(self.as_raw() as _, level))?;
        Ok(())
    }

    /// Retrieves the blend between 3D panning and 2D panning.
    pub fn get_3d_level(&self) -> Result<f32> {
        let mut level = 0.0;
        ffi!(FMOD_Channel_Get3DLevel(self.as_raw() as _, &mut level))?;
        Ok(level)
    }

    /// Sets the minimum and maximum distances used ot calculate the 3D roll-off attenuation.
    pub fn set_3d_min_max_distance(&self, distance: Range<f32>) -> Result {
        ffi!(FMOD_Channel_Set3DMinMaxDistance(
            self.as_raw() as _,
            distance.start,
            distance.end,
        ))?;
        Ok(())
    }

    /// Retrieves the minimum and maximum distances used ot calculate the 3D roll-off attenuation.
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

/// # Panning and level adjustment
impl ChannelControl {
    /// Sets the left/right pan level.
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

/// # Filtering
impl ChannelControl {
    /// Sets the wet / send level for a particular reverb instance.
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

    /// Sets the gain of the dry signal when built in lowpass / distance filtering is applied.
    pub fn set_low_pass_gain(&self, gain: f32) -> Result {
        ffi!(FMOD_Channel_SetLowPassGain(self.as_raw() as _, gain))?;
        Ok(())
    }

    /// Retrieves the gain of the dry signal when built in lowpass / distance filtering is applied.
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
    pub fn set_dsp_index(&self, dsp: &Dsp, index: i32) -> Result {
        ffi!(FMOD_Channel_SetDSPIndex(
            self.as_raw() as _,
            dsp.as_raw(),
            index,
        ))?;
        Ok(())
    }

    /// Retrieves the index of a DSP inside the Channel or ChannelGroup's DSP chain.
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
    pub fn get_dsp_clock(&self) -> Result<u64> {
        let mut dsp_clock = 0;
        ffi!(FMOD_Channel_GetDSPClock(
            self.as_raw() as _,
            &mut dsp_clock,
            ptr::null_mut()
        ))?;
        Ok(dsp_clock)
    }

    /// Retrieves the DSP clock value for the tail DSP node of the parent ChannelGroup.
    pub fn get_parent_dsp_clock(&self) -> Result<u64> {
        let mut parent_clock = 0;
        ffi!(FMOD_Channel_GetDSPClock(
            self.as_raw() as _,
            ptr::null_mut(),
            &mut parent_clock,
        ))?;
        Ok(parent_clock)
    }

    /// Sets a sample accurate start (and/or stop) time relative to the parent ChannelGroup DSP clock.
    pub fn set_delay(&self, dsp_clock: Range<u64>, stop_channels: StopAction) -> Result {
        ffi!(FMOD_Channel_SetDelay(
            self.as_raw() as _,
            dsp_clock.start,
            dsp_clock.end,
            stop_channels as _,
        ))?;
        Ok(())
    }

    /// Retrieves a sample accurate start (and/or stop) time relative to the parent ChannelGroup DSP clock.
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

    /// Adds a sample accurate fade point at a time relative to the parent ChannelGroup DSP clock.
    pub fn add_fade_point(&self, dsp_clock: u64, volume: f32) -> Result {
        ffi!(FMOD_Channel_AddFadePoint(
            self.as_raw() as _,
            dsp_clock,
            volume,
        ))?;
        Ok(())
    }

    /// Adds a volume ramp at the specified time in the future using fade points.
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

/// # General
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
