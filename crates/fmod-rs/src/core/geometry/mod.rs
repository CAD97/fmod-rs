use fmod::{raw::*, *};

/// # Object general.
impl Geometry {
    // add_polygon, set_active, get_active, get_max_polygons, get_num_polygons
    // set_user_data, get_user_data

    raw! {
        /// Frees a geometry object and releases its memory.
        pub unsafe fn raw_release(this: *mut FMOD_GEOMETRY) -> FMOD_RESULT {
            FMOD_Geometry_Release(this)
        }
    }

    // save
}
