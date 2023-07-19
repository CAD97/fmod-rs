use {
    fmod::{raw::*, *},
    std::{
        ops::{Bound, Range, RangeBounds},
        ptr,
    },
};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

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
