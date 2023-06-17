use {
    fmod::{raw::*, utils::catch_user_unwind, *},
    std::{
        ffi::c_void,
        ops::Deref,
        ops::{Bound, RangeBounds, RangeInclusive},
        panic::AssertUnwindSafe,
        ptr,
    },
};

opaque! {
    /// A source of audio signal that connects to the [ChannelGroup] mixing hierarchy.
    ///
    /// Create with [System::play_sound] or [System::play_dsp].
    weak class Channel = FMOD_CHANNEL, FMOD_Channel_*;
}

impl Deref for Channel {
    type Target = ChannelControl;
    fn deref(&self) -> &Self::Target {
        unsafe { ChannelControl::from_raw(self.as_raw() as _) }
    }
}

/// # Playback control
impl Channel {
    /// Sets the frequency or playback rate.
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
    pub fn set_priority(&self, priority: i32) -> Result {
        ffi!(FMOD_Channel_SetPriority(self.as_raw(), priority))?;
        Ok(())
    }

    /// Retrieves the priority used for virtual voice ordering.
    pub fn get_priority(&self) -> Result<i32> {
        let mut priority = 0;
        ffi!(FMOD_Channel_GetPriority(self.as_raw(), &mut priority))?;
        Ok(priority)
    }

    /// Sets the current playback position.
    pub fn set_position(&self, position: u32, pos_type: TimeUnit) -> Result {
        let postype = TimeUnit::into_raw(pos_type);
        ffi!(FMOD_Channel_SetPosition(self.as_raw(), position, postype,))?;
        Ok(())
    }

    /// Retrieves the current playback position.
    pub fn get_position(&self, pos_type: TimeUnit) -> Result<u32> {
        let mut position = 0;
        let postype = TimeUnit::into_raw(pos_type);
        ffi!(FMOD_Channel_GetPosition(
            self.as_raw(),
            &mut position,
            postype,
        ))?;
        Ok(position)
    }

    /// Sets the ChannelGroup this object outputs to.
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
    pub fn set_loop_count(&self, loop_count: i32) -> Result {
        ffi!(FMOD_Channel_SetLoopCount(self.as_raw(), loop_count))?;
        Ok(())
    }

    /// Retrieves the number of times to loop before stopping.
    pub fn get_loop_count(&self) -> Result<i32> {
        let mut loop_count = 0;
        ffi!(FMOD_Channel_GetLoopCount(self.as_raw(), &mut loop_count))?;
        Ok(loop_count)
    }

    /// Sets the loop start and end points.
    pub fn set_loop_points(
        &self,
        loop_points: impl RangeBounds<u32>,
        length_type: TimeUnit,
    ) -> Result {
        let loop_start = match loop_points.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let (loop_end, loop_end_type) = match loop_points.end_bound() {
            Bound::Included(&end) => (end, length_type),
            Bound::Excluded(&end) => (end.saturating_sub(1), length_type),
            Bound::Unbounded => (
                self.get_current_sound()?
                    .get_length(TimeUnit::Pcm)?
                    .saturating_sub(1),
                TimeUnit::Pcm,
            ),
        };
        ffi!(FMOD_Channel_SetLoopPoints(
            self.as_raw(),
            loop_start,
            length_type.into_raw(),
            loop_end,
            loop_end_type.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the loop start and end points.
    pub fn get_loop_points(&self, length_type: TimeUnit) -> Result<RangeInclusive<u32>> {
        let mut start = 0;
        let mut end = 0;
        ffi!(FMOD_Channel_GetLoopPoints(
            self.as_raw(),
            &mut start,
            length_type.into_raw(),
            &mut end,
            length_type.into_raw(),
        ))?;
        Ok(start..=end)
    }
}

/// # Information
impl Channel {
    /// Retrieves whether the Channel is being emulated by the virtual voice system.
    pub fn is_virtual(&self) -> Result<bool> {
        let mut is_virtual = 0;
        ffi!(FMOD_Channel_IsVirtual(self.as_raw(), &mut is_virtual))?;
        Ok(is_virtual != 0)
    }

    /// Retrieves the currently playing Sound.
    pub fn get_current_sound(&self) -> Result<&Sound> {
        let mut sound = ptr::null_mut();
        ffi!(FMOD_Channel_GetCurrentSound(self.as_raw(), &mut sound))?;
        Ok(unsafe { Sound::from_raw(sound) })
    }

    /// Retrieves the index of this object in the System Channel pool.
    pub fn get_index(&self) -> Result<i32> {
        let mut index = 0;
        ffi!(FMOD_Channel_GetIndex(self.as_raw(), &mut index))?;
        Ok(index)
    }
}

// Inherited from ChannelControl
#[doc(hidden)]
impl Channel {
    /// Sets the callback for ChannelControl level notifications.
    pub fn set_callback<C: ChannelCallback>(&self) -> Result {
        ffi!(FMOD_Channel_SetCallback(
            self.as_raw(),
            Some(channel_callback::<C>),
        ))?;
        Ok(())
    }
}

pub trait ChannelCallback {
    /// Called when a sound ends.
    fn end(channel: &Channel) {
        let _ = channel;
    }

    /// Called when a [Channel] is made virtual or real.
    fn virtual_voice(channel: &Channel, is_virtual: bool) {
        let _ = (channel, is_virtual);
    }

    /// Called when a sync point is encountered.
    /// Can be from wav file markers or user added.
    fn sync_point(channel: &Channel, point: i32) {
        let _ = (channel, point);
    }

    /// Called when geometry occlusion values are calculated.
    /// Can be used to clamp or change the value.
    fn occlusion(channel: &Channel, direct: &mut f32, reverb: &mut f32) {
        let _ = (channel, direct, reverb);
    }
}

pub(crate) unsafe extern "system" fn channel_callback<C: ChannelCallback>(
    channelcontrol: *mut FMOD_CHANNELCONTROL,
    controltype: FMOD_CHANNELCONTROL_TYPE,
    callbacktype: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    commanddata1: *mut c_void,
    commanddata2: *mut c_void,
) -> FMOD_RESULT {
    if controltype != FMOD_CHANNELCONTROL_CHANNEL {
        whoops!(no_panic: "channel callback called with channel group");
        return FMOD_ERR_INVALID_PARAM;
    }

    let channel = Channel::from_raw(channelcontrol as *mut FMOD_CHANNEL);
    match callbacktype {
        FMOD_CHANNELCONTROL_CALLBACK_END => catch_user_unwind(|| Ok(C::end(&channel))).into_raw(),
        FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE => {
            let is_virtual = commanddata1 as i32 != 0;
            catch_user_unwind(|| Ok(C::virtual_voice(&channel, is_virtual))).into_raw()
        },
        FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT => {
            let point = commanddata1 as i32;
            catch_user_unwind(|| Ok(C::sync_point(&channel, point))).into_raw()
        },
        FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION => {
            let mut direct = AssertUnwindSafe(&mut *(commanddata1 as *mut f32));
            let mut reverb = AssertUnwindSafe(&mut *(commanddata2 as *mut f32));
            catch_user_unwind(move || Ok(C::occlusion(&channel, &mut direct, &mut reverb)))
                .into_raw()
        },
        _ => {
            whoops!(no_panic: "unknown channel callback type {:?}", callbacktype);
            FMOD_ERR_INVALID_PARAM
        },
    }
}
