use {
    crate::utils::{catch_user_unwind, fmod_get_string},
    fmod::{raw::*, *},
    std::{ffi::c_void, ops::Deref},
};

impl Deref for ChannelGroup {
    type Target = ChannelControl;
    fn deref(&self) -> &Self::Target {
        unsafe { ChannelControl::from_raw(self.as_raw() as _) }
    }
}

/// # General.
impl ChannelGroup {
    /// Retrieves the name set when the group was created.
    pub fn get_name(&self, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_ChannelGroup_GetName(
                    self.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as _,
                ))
            })
        }
    }

    raw! {
        /// Frees the memory for the group.
        ///
        /// Any [`Channel`]s or [`ChannelGroup`]s feeding into this group are moved
        /// to the master [`ChannelGroup`].
        pub unsafe fn raw_release(this: *mut FMOD_CHANNELGROUP) -> FMOD_RESULT {
            FMOD_ChannelGroup_Release(this)
        }
    }

    /// Sets the callback for ChannelGroup level notifications.
    pub fn set_callback<C: ChannelGroupCallback>(&self) -> Result {
        ffi!(FMOD_ChannelGroup_SetCallback(
            self.as_raw(),
            Some(channel_group_callback::<C>),
        ))?;
        Ok(())
    }
}

/// Callback for ChannelGroup notifications.
///
/// Callbacks are called from the game thread when set from the Core API or
/// Studio API in synchronous mode, and from the Studio Update Thread when in
/// default / async mode.
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
    let control_type = ChannelControlType::from_raw(controltype);
    if control_type != ChannelControlType::ChannelGroup {
        whoops!(no_panic: "channel group callback called with channel");
        return Error::InvalidParam.into_raw();
    }

    let callback_type = ChannelControlCallbackType::from_raw(callbacktype);
    let group = ChannelGroup::from_raw(channelcontrol as *mut FMOD_CHANNELGROUP);
    catch_user_unwind(|| match callback_type {
        | ChannelControlCallbackType::End
        | ChannelControlCallbackType::VirtualVoice
        | ChannelControlCallbackType::SyncPoint => {
            whoops!(no_panic: "invalid callback type {:?} for channel group", callbacktype);
            yeet!(Error::InvalidParam)
        },
        ChannelControlCallbackType::Occlusion => {
            let direct = &mut *(commanddata1 as *mut f32);
            let reverb = &mut *(commanddata2 as *mut f32);
            Ok(C::occlusion(group, direct, reverb))
        },
        _ => {
            whoops!(no_panic: "unknown channel callback type {:?}", callback_type);
            yeet!(Error::InvalidParam)
        },
    })
    .into_raw()
}
