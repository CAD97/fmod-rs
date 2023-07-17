use fmod::{raw::*, *};

/// General.
impl Reverb3d {
    // set_3d_attributes, get_3d_attributes, set_properties, get_properties, set_active, get_active
    // set_user_data, get_user_data

    raw! {
        /// Releases the memory for a reverb object and makes it inactive.
        ///
        /// If you release all Reverb3D objects and have not added a new
        /// Reverb3D object, [`System::set_reverb_properties`] should be called
        /// to reset the reverb properties.
        pub unsafe fn raw_release(this: *mut FMOD_REVERB3D) -> FMOD_RESULT {
            FMOD_Reverb3D_Release(this)
        }
    }
}
