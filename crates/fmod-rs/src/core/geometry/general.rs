use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Object general.
impl Geometry {
    /// Adds a polygon.
    ///
    /// All vertices must lay in the same plane otherwise behavior may be
    /// unpredictable. The polygon is assumed to be convex. A non convex
    /// polygon will produce unpredictable behavior. Polygons with zero area
    /// will be ignored.
    ///
    /// Polygons cannot be added if already at the maximum number of polygons
    /// or if the addition of their verticies would result in exceeding the
    /// maximum number of vertices.
    ///
    /// Vertices of an object are in object space, not world space, and so are
    /// relative to the position, or center of the object. See
    /// [`Geometry::set_position`].
    pub fn add_polygon(&self, attrs: PolygonAttributes, vertices: &[Vector]) -> Result<i32> {
        let mut index = 0;
        ffi!(FMOD_Geometry_AddPolygon(
            self.as_raw(),
            attrs.occlusion.direct,
            attrs.occlusion.reverb,
            attrs.double_sided as FMOD_BOOL,
            vertices.len() as i32,
            vertices.as_ptr() as _,
            &mut index,
        ))?;
        Ok(index)
    }

    /// Sets whether an object is processed by the geometry engine.
    pub fn set_active(&self, active: bool) -> Result {
        ffi!(FMOD_Geometry_SetActive(self.as_raw(), active as FMOD_BOOL))?;
        Ok(())
    }

    /// Retrieves whether an object is processed by the geometry engine.
    pub fn get_active(&self) -> Result<bool> {
        let mut active = 0;
        ffi!(FMOD_Geometry_GetActive(self.as_raw(), &mut active))?;
        Ok(active != 0)
    }

    /// Retrieves the maximum number of polygons allocatable for this object.
    ///
    /// The maximum number was set with [`System::create_geometry`].
    pub fn get_max_polygons(&self) -> Result<i32> {
        let mut max_polygons = 0;
        ffi!(FMOD_Geometry_GetMaxPolygons(
            self.as_raw(),
            &mut max_polygons,
            ptr::null_mut(),
        ))?;
        Ok(max_polygons)
    }

    /// Retrieves the maximum number of vertices allocatable for this object.
    ///
    /// The maximum number was set with [`System::create_geometry`].
    pub fn get_max_vertices(&self) -> Result<i32> {
        let mut max_vertices = 0;
        ffi!(FMOD_Geometry_GetMaxPolygons(
            self.as_raw(),
            ptr::null_mut(),
            &mut max_vertices,
        ))?;
        Ok(max_vertices)
    }

    pub fn get_num_polygons(&self) -> Result<i32> {
        let mut num_polygons = 0;
        ffi!(FMOD_Geometry_GetNumPolygons(
            self.as_raw(),
            &mut num_polygons,
        ))?;
        Ok(num_polygons)
    }

    // set_user_data, get_user_data

    raw! {
        /// Frees a geometry object and releases its memory.
        pub unsafe fn raw_release(this: *mut FMOD_GEOMETRY) -> FMOD_RESULT {
            FMOD_Geometry_Release(this)
        }
    }

    /// Saves the geometry object as a serialized binary block to a user memory
    /// buffer.
    ///
    /// Typical usage of this function is to call it twice - once to get the
    /// size of the data, then again to write the data to your pointer.
    ///
    /// The data can be saved to a file if required and loaded later with
    /// [`System::load_geometry`].
    pub fn save(&self, data: &mut Vec<u8>) -> Result {
        let mut size = 0;
        ffi!(FMOD_Geometry_Save(
            self.as_raw(),
            ptr::null_mut(),
            &mut size,
        ))?;
        data.clear();
        data.reserve(ix!(size));
        ffi!(FMOD_Geometry_Save(
            self.as_raw(),
            data.spare_capacity_mut().as_mut_ptr() as _,
            &mut size,
        ))?;
        unsafe { data.set_len(ix!(size)) };
        Ok(())
    }
}
