use crate::{raw::*, Error, FmodResource, Result};

opaque! {
    class Dsp;
}

impl Dsp {
    raw! {
        #[allow(clippy::missing_safety_doc)]
        pub unsafe fn from_raw(raw: *mut FMOD_DSP) -> &'static Dsp {
            &*(raw as *mut _)
        }

        pub fn as_raw(&self) -> *mut FMOD_DSP {
            self as *const _ as *const _ as *mut _
        }
    }
}

unsafe impl FmodResource for Dsp {
    type Raw = FMOD_DSP;

    unsafe fn release(this: *mut FMOD_DSP) -> Result<()> {
        let result = FMOD_DSP_Release(this);
        if let Some(error) = Error::from_raw(result) {
            Err(error)
        } else {
            Ok(())
        }
    }
}
