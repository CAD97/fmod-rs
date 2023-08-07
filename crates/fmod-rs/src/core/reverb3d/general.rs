use fmod::{raw::*, *};

/// General.
impl Reverb3d {
    /// Sets the 3D attributes of a reverb sphere.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    ///
    /// When the position of the listener is less than max distance away from
    /// the position of one or more reverb objects, the listener's 3D reverb
    /// properties are a weighted combination of those reverb objects.
    /// Otherwise, the reverb dsp will use the global reverb settings.
    pub fn set_3d_attributes(&self, attributes: ReverbAttributes3d) -> Result {
        ffi!(FMOD_Reverb3D_Set3DAttributes(
            self.as_raw(),
            attributes.position.as_raw(),
            attributes.min_distance,
            attributes.max_distance,
        ))?;
        Ok(())
    }

    /// Retrieves the 3D attributes of a reverb sphere.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    pub fn get_3d_attributes(&self) -> Result<ReverbAttributes3d> {
        let mut attributes = ReverbAttributes3d::default();
        ffi!(FMOD_Reverb3D_Get3DAttributes(
            self.as_raw(),
            attributes.position.as_raw_mut(),
            &mut attributes.min_distance,
            &mut attributes.max_distance,
        ))?;
        Ok(attributes)
    }

    /// Sets the environmental properties of a reverb sphere.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    ///
    /// Reverb presets are available as associated constants to
    /// [ReverbProperties](ReverbProperties#impl-ReverbProperties-1).
    pub fn set_properties(&self, properties: &ReverbProperties) -> Result {
        ffi!(FMOD_Reverb3D_SetProperties(
            self.as_raw(),
            properties.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the environmental properties of a reverb sphere.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    pub fn get_properties(&self) -> Result<ReverbProperties> {
        let mut properties = ReverbProperties::default();
        ffi!(FMOD_Reverb3D_GetProperties(
            self.as_raw(),
            properties.as_raw_mut(),
        ))?;
        Ok(properties)
    }

    /// Sets the active state.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    pub fn set_active(&self, active: bool) -> Result {
        ffi!(FMOD_Reverb3D_SetActive(self.as_raw(), active as FMOD_BOOL))?;
        Ok(())
    }

    /// Retrieves the active state.
    ///
    /// See the [3D Reverb] guide for more information.
    ///
    /// [3D Reverb]: https://fmod.com/docs/2.02/api/white-papers-3d-reverb.html
    pub fn get_active(&self) -> Result<bool> {
        let mut active = 0;
        ffi!(FMOD_Reverb3D_GetActive(self.as_raw(), &mut active))?;
        Ok(active != 0)
    }

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

/// Position and distance range of a reverb object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ReverbAttributes3d {
    /// Position in 3D space representing the center of the reverb.
    pub position: Vector,
    /// Distance from the centerpoint within which the reverb will have full effect.
    pub min_distance: f32,
    /// Distance from the centerpoint beyond which the reverb will have no effect.
    pub max_distance: f32,
}
