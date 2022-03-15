use crate::{raw::*, Error, FmodResource};

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

    unsafe fn release(this: *mut Self) {
        let result = FMOD_DSP_Release(this as *mut _);
        if let Some(error) = Error::from_raw(result) {
            panic!("FMOD error releasing DSP: {error}");
        }
    }
}
