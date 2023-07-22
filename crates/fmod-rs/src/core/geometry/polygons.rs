use fmod::{raw::*, *};

/// # Polygons.
impl Geometry {
    /// Sets individual attributes for a polygon inside a geometry object.
    pub fn set_polygon_attributes(&self, index: i32, attributes: PolygonAttributes) -> Result {
        ffi!(FMOD_Geometry_SetPolygonAttributes(
            self.as_raw(),
            index,
            attributes.occlusion.direct,
            attributes.occlusion.reverb,
            attributes.double_sided as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Retrieves the attributes for a polygon.
    pub fn get_polygon_attributes(&self, index: i32) -> Result<PolygonAttributes> {
        let mut occlusion = Occlusion::default();
        let mut double_sided = 0;
        ffi!(FMOD_Geometry_GetPolygonAttributes(
            self.as_raw(),
            index,
            &mut occlusion.direct,
            &mut occlusion.reverb,
            &mut double_sided,
        ))?;
        Ok(PolygonAttributes {
            occlusion,
            double_sided: double_sided != 0,
        })
    }

    /// Alters the position of a polygon's vertex inside a geometry object.
    ///
    /// Vertices are relative to the position of the object. See
    /// [`Geometry::set_position`].
    ///
    /// There may be some significant overhead with this function as it may
    /// cause some reconfiguration of internal data structures used to speed up
    /// sound-ray testing.
    ///
    /// You may get better results if you want to modify your object by using
    /// [`Geometry::set_position`], [`Geometry::set_scale`] and
    /// [`Geometry::set_rotation`].
    pub fn set_polygon_vertex(&self, index: i32, vertex_index: i32, vertex: &Vector) -> Result {
        ffi!(FMOD_Geometry_SetPolygonVertex(
            self.as_raw(),
            index,
            vertex_index,
            vertex.as_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the position of a vertex.
    ///
    /// Vertices are relative to the position of the object. See
    /// [`Geometry::set_position`].
    pub fn get_polygon_vertex(&self, index: i32, vertex_index: i32) -> Result<Vector> {
        let mut vertex = Vector::default();
        ffi!(FMOD_Geometry_GetPolygonVertex(
            self.as_raw(),
            index,
            vertex_index,
            vertex.as_raw_mut(),
        ))?;
        Ok(vertex)
    }
}

/// Attributes for a polygon inside a geometry object.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PolygonAttributes {
    /// Occlusion factor of the polygon.
    pub occlusion: Occlusion,
    /// If a polygon is single sided, the winding of the polygon (which
    /// determines the polygon's normal) determines which side of the polygon
    /// will cause occlusion.
    pub double_sided: bool,
}
