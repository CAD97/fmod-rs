use {
    fmod::{
        raw::*,
        utils::{catch_user_unwind, fmod_get_string},
        *,
    },
    std::{ffi::c_void, ops::Deref, panic::AssertUnwindSafe, ptr},
};

opaque! {
    /// A submix in the mixing hierarchy akin to a bus that can contain both [Channel] and [ChanelGroup] objects.
    ///
    /// Create with [System::create_channel_group].
    class ChannelGroup = FMOD_CHANNELGROUP, FMOD_ChannelGroup_*;
}

impl Deref for ChannelGroup {
    type Target = ChannelControl;
    fn deref(&self) -> &Self::Target {
        unsafe { ChannelControl::from_raw(self.as_raw() as _) }
    }
}

/// # Channel management
impl ChannelGroup {
    /// Retrieves the number of Channels that feed into this group.
    pub fn get_num_channels(&self) -> Result<i32> {
        let mut num_channels = 0;
        ffi!(FMOD_ChannelGroup_GetNumChannels(
            self.as_raw(),
            &mut num_channels
        ))?;
        Ok(num_channels)
    }

    /// Retrieves the Channel at the specified index in the list of Channel inputs.
    pub fn get_channel(&self, index: i32) -> Result<&Channel> {
        let mut channel = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_GetChannel(
            self.as_raw(),
            index,
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }
}

/// # ChannelGroup management
impl ChannelGroup {
    // TODO: allow setting propagated_dsp_clock = false somehow
    /// Adds a ChannelGroup as an input to this group.
    pub fn add_group(&self, group: &ChannelGroup) -> Result<&DspConnection> {
        let mut connection = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_AddGroup(
            self.as_raw(),
            group.as_raw(),
            /* propagated_dsp_clock */ true as _,
            &mut connection,
        ))?;
        Ok(unsafe { DspConnection::from_raw(connection) })
    }

    /// Retrieves the number of ChannelGroups that feed into this group.
    pub fn get_num_groups(&self) -> Result<i32> {
        let mut num_groups = 0;
        ffi!(FMOD_ChannelGroup_GetNumGroups(
            self.as_raw(),
            &mut num_groups
        ))?;
        Ok(num_groups)
    }

    /// Retrieves the ChannelGroup at the specified index in the list of group inputs.
    pub fn get_group(&self, index: i32) -> Result<&ChannelGroup> {
        let mut group = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_GetGroup(self.as_raw(), index, &mut group))?;
        Ok(unsafe { ChannelGroup::from_raw(group) })
    }

    /// Retrieves the ChannelGroup this object outputs to.
    pub fn get_parent_group(&self) -> Result<Option<&ChannelGroup>> {
        let mut group = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_GetParentGroup(self.as_raw(), &mut group))?;
        if group.is_null() {
            Ok(None)
        } else {
            Ok(Some(unsafe { ChannelGroup::from_raw(group) }))
        }
    }
}

/// # General
impl ChannelGroup {
    /// Retrieves the name set when the group was created.
    pub fn get_name(&self, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_ChannelGroup_GetName(
                    self.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as _
                ))
            })
        }
    }
}

// Inherited from ChannelControl
#[doc(hidden)]
impl ChannelGroup {
    /// Sets the callback for ChannelControl level notifications.
    pub fn set_callback<C: ChannelGroupCallback>(&self) -> Result {
        ffi!(FMOD_ChannelGroup_SetCallback(
            self.as_raw(),
            Some(channel_group_callback::<C>),
        ))?;
        Ok(())
    }
}

pub trait ChannelGroupCallback {
    /// Called when geometry occlusion values are calculated.
    /// Can be used to clamp or change the value.
    fn occlusion(channel: &ChannelGroup, direct: &mut f32, reverb: &mut f32) {
        let _ = (channel, direct, reverb);
    }
}

pub(crate) unsafe extern "system" fn channel_group_callback<C: ChannelGroupCallback>(
    channelcontrol: *mut FMOD_CHANNELCONTROL,
    controltype: FMOD_CHANNELCONTROL_TYPE,
    callbacktype: FMOD_CHANNELCONTROL_CALLBACK_TYPE,
    commanddata1: *mut c_void,
    commanddata2: *mut c_void,
) -> FMOD_RESULT {
    if controltype != FMOD_CHANNELCONTROL_CHANNELGROUP {
        whoops!(no_panic: "channel group callback called with channel");
        return FMOD_ERR_INVALID_PARAM;
    }

    let group = AssertUnwindSafe(ChannelGroup::from_raw(
        channelcontrol as *mut FMOD_CHANNELGROUP,
    ));
    match callbacktype {
        | FMOD_CHANNELCONTROL_CALLBACK_END
        | FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE
        | FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT => {
            whoops!(no_panic: "invalid callback type {:?} for channel group", callbacktype);
            FMOD_ERR_INVALID_PARAM
        },
        FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION => {
            let mut direct = AssertUnwindSafe(&mut *(commanddata1 as *mut f32));
            let mut reverb = AssertUnwindSafe(&mut *(commanddata2 as *mut f32));
            catch_user_unwind(move || Ok(C::occlusion(&group, &mut direct, &mut reverb))).into_raw()
        },
        _ => {
            whoops!(no_panic: "unknown channel callback type {:?}", callbacktype);
            FMOD_ERR_INVALID_PARAM
        },
    }
}
