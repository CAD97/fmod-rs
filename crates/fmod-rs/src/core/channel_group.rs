use crate::{raw::*, Error, FmodResource, Result};

opaque! {
    class ChannelGroup;
}

impl ChannelGroup {
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut FMOD_CHANNELGROUP) -> &'static ChannelGroup {
            &*(raw as *mut _)
        }

        pub fn as_raw(&self) -> *mut FMOD_CHANNELGROUP {
            self as *const _ as *const _ as *mut _
        }
    }
}

unsafe impl FmodResource for ChannelGroup {
    type Raw = FMOD_CHANNELGROUP;

    unsafe fn release(this: *mut FMOD_CHANNELGROUP) -> Result<()> {
        let result = FMOD_ChannelGroup_Release(this);
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }
}
