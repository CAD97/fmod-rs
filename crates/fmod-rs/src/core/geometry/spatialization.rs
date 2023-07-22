use fmod::{raw::*, *};

/// # Object Spatialization.
impl Geometry {
    /// Sets the 3D position of the object.
    ///
    /// Position is in world space.
    pub fn set_position(&self, position: &Vector) -> Result {
        ffi!(FMOD_Geometry_SetPosition(self.as_raw(), position.as_raw()))?;
        Ok(())
    }

    /// Retrieves the 3D position of the object.
    ///
    /// Position is in world space.
    pub fn get_position(&self) -> Result<Vector> {
        let mut position = Vector::default();
        ffi!(FMOD_Geometry_GetPosition(
            self.as_raw(),
            position.as_raw_mut()
        ))?;
        Ok(position)
    }

    /// Sets the 3D orientation of the object.
    ///
    /// See remarks in [`System::set_3d_listener_attributes`] for more
    /// description on forward and up vectors.
    pub fn set_rotation(&self, orientation: &Orientation3d) -> Result {
        ffi!(FMOD_Geometry_SetRotation(
            self.as_raw(),
            orientation.forward.as_raw(),
            orientation.up.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the 3D orientation of the object.
    pub fn get_rotation(&self) -> Result<Orientation3d> {
        let mut orientation = Orientation3d::default();
        ffi!(FMOD_Geometry_GetRotation(
            self.as_raw(),
            orientation.forward.as_raw_mut(),
            orientation.up.as_raw_mut(),
        ))?;
        Ok(orientation)
    }

    /// Sets the 3D scale of the object.
    ///
    /// An object can be scaled/warped in all 3 dimensions separately using this
    /// function without having to modify polygon data.
    pub fn set_scale(&self, scale: &Vector) -> Result {
        ffi!(FMOD_Geometry_SetScale(self.as_raw(), scale.as_raw()))?;
        Ok(())
    }

    /// Retrieves the 3D scale of the object.
    pub fn get_scale(&self) -> Result<Vector> {
        let mut scale = Vector::default();
        ffi!(FMOD_Geometry_GetScale(self.as_raw(), scale.as_raw_mut()))?;
        Ok(scale)
    }
}
