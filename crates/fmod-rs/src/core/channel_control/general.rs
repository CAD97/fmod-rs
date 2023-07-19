use {
    fmod::{raw::*, *},
    std::{ffi::c_void, ptr},
};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

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

#[cfg(not(feature = "unstable"))]
/// Callback for Channel and ChannelGroup notifications.
///
/// Callbacks are called from the game thread when set from the Core API or
/// Studio API in synchronous mode, and from the Studio Update Thread when in
/// default / async mode.
pub trait ChannelControlCallback: ChannelCallback + ChannelGroupCallback {}
#[cfg(not(feature = "unstable"))]
impl<C: ChannelCallback + ChannelGroupCallback> ChannelControlCallback for C {}

#[cfg(feature = "unstable")]
/// Callback for Channel and ChannelGroup notifications.
///
/// Callbacks are called from the game thread when set from the Core API or
/// Studio API in synchronous mode, and from the Studio Update Thread when in
/// default / async mode.
pub trait ChannelControlCallback = ChannelCallback + ChannelGroupCallback;

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

raw! {
    enum_struct! {
        /// Identifier used to distinguish between Channel and ChannelGroup in the ChannelControl callback.
        pub enum ChannelControlType: FMOD_CHANNELCONTROL_TYPE {
            /// Type representing [Channel]
            Channel      = FMOD_CHANNELCONTROL_CHANNEL,
            /// Type representing [ChannelGroup]
            ChannelGroup = FMOD_CHANNELCONTROL_CHANNELGROUP,
        }
    }
}

raw! {
    enum_struct! {
        /// Types of callbacks called by Channels and ChannelGroups.
        pub enum ChannelControlCallbackType: FMOD_CHANNELCONTROL_CALLBACK_TYPE {
            /// Called when a sound ends. Supported by [Channel] only.
            End          = FMOD_CHANNELCONTROL_CALLBACK_END,
            /// Called when a [Channel] is made virtual or real. Supported by [Channel] objects only.
            ///
            /// - `command_data_1`: (int) 0 represents 'virtual to real' and 1 represents 'real to virtual'.
            VirtualVoice = FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE,
            /// Called when a syncpoint is encountered. Can be from wav file markers or user added. Supported by [Channel] only.
            ///
            /// - `command_data_1`: (int) representing the index of the sync point for use with [Sound::get_sync_point_info].
            SyncPoint    = FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT,
            /// Called when geometry occlusion values are calculated. Can be used to clamp or change the value. Supported by [Channel] and [ChannelGroup].
            Occlusion    = FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION,
        }
    }
}
