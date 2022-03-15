#[cfg(feature = "raw")]
pub use __shenanigans::raw;
#[cfg(not(feature = "raw"))]
#[allow(unused)]
use __shenanigans::raw;

mod __shenanigans {
    pub mod raw {
        #[doc(inline)]
        pub use fmod_core_sys::*;

        #[doc(inline)]
        #[cfg(feature = "studio")]
        pub use fmod_studio_sys::*;
    }
}
