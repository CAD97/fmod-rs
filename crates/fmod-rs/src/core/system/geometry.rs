use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Geometry management.
impl System {
    /// Geometry creation function. This function will create a base geometry
    /// object which can then have polygons added to it.
    ///
    /// Polygons can be added to a geometry object using
    /// [`Geometry::add_polygon`]. For best efficiency, avoid overlapping of
    /// polygons and long thin polygons.
    ///
    /// A geometry object stores its polygons in a group to allow optimization
    /// for line testing, insertion and updating of geometry in real-time.
    /// Geometry objects also allow for efficient rotation, scaling and
    /// translation of groups of polygons.
    ///
    /// It is important to set the value of max_world_size to an appropriate
    /// value using [`System::set_geometry_settings`].
    pub fn create_geometry(
        &self,
        max_polygons: i32,
        max_vertices: i32,
    ) -> Result<Handle<'_, Geometry>> {
        let mut geometry = ptr::null_mut();
        ffi!(FMOD_System_CreateGeometry(
            self.as_raw(),
            max_polygons,
            max_vertices,
            &mut geometry,
        ))?;
        Ok(unsafe { Handle::new(geometry) })
    }

    /// Sets the maximum world size for the geometry engine for performance /
    /// precision reasons.
    ///
    /// FMOD uses an efficient spatial partitioning system to store polygons for
    /// ray casting purposes. The maximum size of the world should be set to
    /// allow processing within a known range. Outside of this range, objects
    /// and polygons will not be processed as efficiently. Excessive world size
    /// settings can also cause loss of precision and efficiency.
    ///
    /// Setting `max_world_size` should be done first before creating any
    /// geometry. It can be done any time afterwards but may be slow in this
    /// case.
    pub fn set_geometry_settings(&self, max_world_size: f32) -> Result {
        ffi!(FMOD_System_SetGeometrySettings(
            self.as_raw(),
            max_world_size,
        ))?;
        Ok(())
    }

    /// Retrieves the maximum world size for the geometry engine.
    ///
    /// FMOD uses an efficient spatial partitioning system to store polygons for
    /// ray casting purposes. The maximum size of the world should be set to
    /// allow processing within a known range. Outside of this range, objects
    /// and polygons will not be processed as efficiently. Excessive world size
    /// settings can also cause loss of precision and efficiency.
    pub fn get_geometry_settings(&self) -> Result<f32> {
        let mut max_world_size = 0.0;
        ffi!(FMOD_System_GetGeometrySettings(
            self.as_raw(),
            &mut max_world_size,
        ))?;
        Ok(max_world_size)
    }

    /// Creates a geometry object from a block of memory which contains
    /// pre-saved geometry data from [`Geometry::save`].
    ///
    /// This function avoids the need to manually create and add geometry for
    /// faster start time.
    pub fn load_geometry(&self, data: &[u8]) -> Result<Handle<'_, Geometry>> {
        let mut geometry = ptr::null_mut();
        ffi!(FMOD_System_LoadGeometry(
            self.as_raw(),
            data.as_ptr().cast(),
            data.len() as _,
            &mut geometry,
        ))?;
        Ok(unsafe { Handle::new(geometry) })
    }

    /// Calculates geometry occlusion between a listener and a sound source.
    ///
    /// If single sided polygons have been created, it is important to get the
    /// source and listener positions around the right way, as the occlusion
    /// from point A to point B may not be the same as the occlusion from point
    /// B to point A.
    pub fn get_geometry_occlusion(&self, listener: &Vector, source: &Vector) -> Result<Occlusion> {
        let mut direct = 0.0;
        let mut reverb = 0.0;
        ffi!(FMOD_System_GetGeometryOcclusion(
            self.as_raw(),
            listener.as_raw(),
            source.as_raw(),
            &mut direct,
            &mut reverb,
        ))?;
        Ok(Occlusion { direct, reverb })
    }
}
