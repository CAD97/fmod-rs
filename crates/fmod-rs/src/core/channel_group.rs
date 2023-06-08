use {
    fmod::{raw::*, utils::catch_user_unwind, *},
    std::{ffi::c_void, ops::Deref, panic::AssertUnwindSafe},
};

opaque!(class ChannelGroup = FMOD_CHANNELGROUP, FMOD_ChannelGroup_*);

impl Deref for ChannelGroup {
    type Target = ChannelControl;
    fn deref(&self) -> &Self::Target {
        unsafe { ChannelControl::from_raw(self.as_raw() as _) }
    }
}

/// General control functionality.
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
            return FMOD_ERR_INVALID_PARAM;
        },
        FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION => {
            let mut direct = AssertUnwindSafe(&mut *(commanddata1 as *mut f32));
            let mut reverb = AssertUnwindSafe(&mut *(commanddata2 as *mut f32));
            catch_user_unwind(move || C::occlusion(&group, &mut direct, &mut reverb));
        },
        _ => {
            whoops!(no_panic: "unknown channel callback type {:?}", callbacktype);
            return FMOD_ERR_INVALID_PARAM;
        },
    }

    FMOD_OK
}
