use {
    fmod::{raw::*, *},
    std::{
        ops::{Bound, RangeBounds, RangeInclusive},
        ptr,
    },
};

/// # Playback control.
impl Channel {
    /// Sets the frequency or playback rate.
    ///
    /// Default frequency is determined by the audio format of the [`Sound`]
    /// or [`Dsp`].
    ///
    /// Sounds opened as [`Mode::CreateSample`] (not [`Mode::CreateStream`] or
    /// [`Mode::CreateCompressedSample`]) can be played backwards by giving a
    /// negative frequency.
    pub fn set_frequency(&self, frequency: f32) -> Result {
        ffi!(FMOD_Channel_SetFrequency(self.as_raw(), frequency))?;
        Ok(())
    }

    /// Retrieves the playback frequency or playback rate.
    pub fn get_frequency(&self) -> Result<f32> {
        let mut frequency = 0.0;
        ffi!(FMOD_Channel_GetFrequency(self.as_raw(), &mut frequency))?;
        Ok(frequency)
    }

    /// Sets the priority used for virtual voice ordering.
    ///
    /// 0 represents most important and 256 represents least important.
    /// The default priority is 128.
    ///
    /// Priority is used as a coarse grain control for the virtual voice system,
    /// lower priority [`Channel`]s will always be stolen before higher ones.
    /// For [`Channel`]s of equal priority, those with the quietest
    /// [`ChannelControl::get_audibility`] value will be stolen first.
    ///
    /// See the [Virtual Voices] guide for more information.
    ///
    /// [Virtual Voices]: https://fmod.com/docs/2.02/api/white-papers-virtual-voices.html
    pub fn set_priority(&self, priority: i32) -> Result {
        ffi!(FMOD_Channel_SetPriority(self.as_raw(), priority))?;
        Ok(())
    }

    /// Retrieves the priority used for virtual voice ordering.
    ///
    /// 0 represents most important and 256 represents least important.
    /// The default priority is 128.
    ///
    /// Priority is used as a coarse grain control for the virtual voice system,
    /// lower priority [`Channel`]s will always be stolen before higher ones.
    /// For [`Channel`]s of equal priority, those with the quietest
    /// [`ChannelControl::get_audibility`] value will be stolen first.
    ///
    /// See the [Virtual Voices] guide for more information.
    ///
    /// [Virtual Voices]: https://fmod.com/docs/2.02/api/white-papers-virtual-voices.html
    pub fn get_priority(&self) -> Result<i32> {
        let mut priority = 0;
        ffi!(FMOD_Channel_GetPriority(self.as_raw(), &mut priority))?;
        Ok(priority)
    }

    /// Sets the current playback position.
    ///
    /// Certain [`TimeUnit`] types are always available: [`TimeUnit::Pcm`],
    /// [`TimeUnit::PcmBytes`] and [`TimeUnit::Ms`]. The others are format
    /// specific such as [`TimeUnit::ModOrder`] / [`TimeUnit::ModRow`] /
    /// [`TimeUnit::ModPattern`] which is specific to files of type MOD / S3M /
    /// XM / IT.
    ///
    /// If playing a [`Sound`] created with [`System::create_stream`] or
    /// [`Mode::CreateStream`] changing the position may cause a slow reflush
    /// operation while the file seek and decode occurs. You can avoid this by
    /// creating the stream with [`Mode::NonBlocking`]. This will cause the
    /// stream to go into [`OpenState::SetPosition`] state (see
    /// [`Sound::get_open_state`]) and [`Sound`] commands will return
    /// [`Error::NotReady`]. [`Channel::get_position`] will also not update
    /// until this non-blocking set position operation has completed.
    ///
    /// Using a VBR source that does not have an associated seek table or seek
    /// information (such as MP3 or MOD/S3M/XM/IT) may cause inaccurate seeking
    /// if you specify [`TimeUnit::Ms`] or [`TimeUnit::Pcm`. If you want FMOD
    /// to create a PCM vs bytes seek table so that seeking is accurate, you
    /// will have to specify [`Mode::AccurateTime`] when loading or opening the
    /// sound. This means there is a slight delay as FMOD scans the whole file
    /// when loading the sound to create this table.
    pub fn set_position(&self, position: Time) -> Result {
        ffi!(FMOD_Channel_SetPosition(
            self.as_raw(),
            position.value,
            position.unit.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the current playback position.
    ///
    /// Certain [`TimeUnit`] types are always available: [`TimeUnit::Pcm`],
    /// [`TimeUnit::PcmBytes`] and [`TimeUnit::Ms`]. The others are format
    /// specific such as [`TimeUnit::ModOrder`] / [`TimeUnit::ModRow`] /
    /// [`TimeUnit::ModPattern`] which is specific to files of type MOD / S3M /
    /// XM / IT.
    ///
    /// If [`TimeUnit::Ms`] or [`TimeUnit::PcmBytes`] are used, the value is
    /// internally converted from [`TimeUnit::Pcm`], so the retrieved value may
    /// not exactly match the set value.
    pub fn get_position(&self, unit: TimeUnit) -> Result<u32> {
        let mut position = 0;
        ffi!(FMOD_Channel_GetPosition(
            self.as_raw(),
            &mut position,
            unit.into_raw(),
        ))?;
        Ok(position)
    }

    /// Sets the ChannelGroup this object outputs to.
    ///
    /// A [`ChannelGroup`] may contain many Channels.
    ///
    /// [`Channel`]s may only output to a single [`ChannelGroup`]. This
    /// operation will remove it from the previous group first.
    pub fn set_channel_group(&self, channel_group: &ChannelGroup) -> Result {
        ffi!(FMOD_Channel_SetChannelGroup(
            self.as_raw(),
            channel_group.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the ChannelGroup this object outputs to.
    pub fn get_channel_group(&self) -> Result<&ChannelGroup> {
        let mut channel_group = ptr::null_mut();
        ffi!(FMOD_Channel_GetChannelGroup(
            self.as_raw(),
            &mut channel_group,
        ))?;
        Ok(unsafe { ChannelGroup::from_raw(channel_group) })
    }

    /// Sets the number of times to loop before stopping.
    ///
    /// 0 represents "oneshot", 1 represents "loop once then stop" and -1
    /// represents "loop forever".
    ///
    /// The 'mode' of the [`Sound`] or [`Channel`] must be [`Mode::LoopNormal`]
    /// or [`Mode::LoopBidi`] for this function to work.
    pub fn set_loop_count(&self, loop_count: i32) -> Result {
        ffi!(FMOD_Channel_SetLoopCount(self.as_raw(), loop_count))?;
        Ok(())
    }

    /// Retrieves the number of times to loop before stopping.
    ///
    /// 0 represents "oneshot", 1 represents "loop once then stop" and -1
    /// represents "loop forever".
    ///
    /// This is the _current_ loop countdown value that will decrement as it
    /// plays until reaching 0. Reset with [`Channel::set_loop_count`].
    pub fn get_loop_count(&self) -> Result<i32> {
        let mut loop_count = 0;
        ffi!(FMOD_Channel_GetLoopCount(self.as_raw(), &mut loop_count))?;
        Ok(loop_count)
    }

    /// Sets the loop start and end points.
    ///
    /// Loop points may only be set on a [`Channel`] playing a [`Sound`], not a
    /// [`Channel`] playing a [`Dsp`] (See [`System::play_dsp`]).
    ///
    /// Valid [`TimeUnit`] types are [`TimeUnit::Pcm`], [`TimeUnit::Ms`],
    /// [`TimeUnit::PcmBytes`]. Any other time units return [`Error::Format`].
    /// If [`TimeUnit::Ms`] or [`TimeUnit::PcmBytes`], the value is internally
    /// converted to [`TimeUnit::Pcm`].
    ///
    /// The [`Channel`]'s mode must be set to [`Mode::LoopNormal`] or
    /// [`Mode::LoopBidi`] for loop points to affect playback.
    pub fn set_loop_points(&self, loop_points: impl RangeBounds<Time>) -> Result {
        let loop_start = match loop_points.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => Time {
                value: start.value.saturating_add(1),
                ..start
            },
            Bound::Unbounded => Time::pcm(0),
        };
        let loop_end = match loop_points.end_bound() {
            Bound::Included(&end) => end,
            Bound::Excluded(&end) => Time {
                value: end.value.saturating_sub(1),
                ..end
            },
            Bound::Unbounded => {
                if let Some(sound) = self.get_current_sound()? {
                    Time::pcm(sound.get_length(TimeUnit::Pcm)?.saturating_sub(1))
                } else {
                    loop_start
                }
            },
        };
        ffi!(FMOD_Channel_SetLoopPoints(
            self.as_raw(),
            loop_start.value,
            loop_start.unit.into_raw(),
            loop_end.value,
            loop_end.unit.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the loop start and end points.
    ///
    /// Valid [`TimeUnit`] types are [`TimeUnit::Pcm`], [`TimeUnit::Ms`],
    /// [`TimeUnit::PcmBytes`]. Any other time units return [`Error::Format`].
    /// If [`TimeUnit::Ms`] or [`TimeUnit::PcmBytes`] are used, the value is
    /// internally converted from [`TimeUnit::Pcm`], so the retrieved value may
    /// not exactly match the set value.
    pub fn get_loop_points(&self, unit: TimeUnit) -> Result<RangeInclusive<u32>> {
        let mut start = 0;
        let mut end = 0;
        ffi!(FMOD_Channel_GetLoopPoints(
            self.as_raw(),
            &mut start,
            unit.into_raw(),
            &mut end,
            unit.into_raw(),
        ))?;
        Ok(start..=end)
    }
}
