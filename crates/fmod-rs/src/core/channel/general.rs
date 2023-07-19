use {
    crate::utils::catch_user_unwind,
    fmod::{raw::*, *},
    std::{ffi::c_void, ops::Deref},
};

impl Deref for Channel {
    type Target = ChannelControl;
    fn deref(&self) -> &Self::Target {
        unsafe { ChannelControl::from_raw(self.as_raw() as _) }
    }
}

// General.
impl Channel {
    /// Sets the callback for Channel level notifications.
    pub fn set_callback<C: ChannelCallback>(&self) -> Result {
        ffi!(FMOD_Channel_SetCallback(
            self.as_raw(),
            Some(channel_callback::<C>),
        ))?;
        Ok(())
    }
}

/// Callback for Channel notifications.
///
/// Callbacks are called from the game thread when set from the Core API or
/// Studio API in synchronous mode, and from the Studio Update Thread when in
/// default / async mode.
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
    let control_type = ChannelControlType::from_raw(controltype);
    if control_type != ChannelControlType::Channel {
        whoops!(no_panic: "channel callback called with channel group");
        return Error::InvalidParam.into_raw();
    }

    let callback_type = ChannelControlCallbackType::from_raw(callbacktype);
    let channel = Channel::from_raw(channelcontrol as *mut FMOD_CHANNEL);
    catch_user_unwind(|| match callback_type {
        ChannelControlCallbackType::End => Ok(C::end(channel)),
        ChannelControlCallbackType::VirtualVoice => {
            let is_virtual = commanddata1 as i32 != 0;
            Ok(C::virtual_voice(channel, is_virtual))
        },
        ChannelControlCallbackType::SyncPoint => {
            let point = commanddata1 as i32;
            Ok(C::sync_point(channel, point))
        },
        ChannelControlCallbackType::Occlusion => {
            let direct = &mut *(commanddata1 as *mut f32);
            let reverb = &mut *(commanddata2 as *mut f32);
            Ok(C::occlusion(channel, direct, reverb))
        },
        _ => {
            whoops!(no_panic: "unknown channel callback type {:?}", callback_type);
            yeet!(Error::InvalidParam)
        },
    })
    .into_raw()
}
