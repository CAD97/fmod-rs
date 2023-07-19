use {
    crate::utils::fmod_get_string,
    fmod::{raw::*, *},
    std::ptr,
};

/// # Synchronization / markers.
impl Sound {
    /// Retrieve a sync point.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn get_sync_point(&self, index: i32) -> Result<&SyncPoint> {
        let mut point = ptr::null_mut();
        ffi!(FMOD_Sound_GetSyncPoint(self.as_raw(), index, &mut point))?;
        Ok(unsafe { SyncPoint::from_raw(point) })
    }

    /// Retrieves information on an embedded sync point.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn get_sync_point_name(&self, sync_point: &SyncPoint, name: &mut String) -> Result {
        unsafe {
            fmod_get_string(name, |buf| {
                ffi!(FMOD_Sound_GetSyncPointInfo(
                    self.as_raw(),
                    sync_point.as_raw(),
                    buf.as_mut_ptr().cast(),
                    buf.len() as i32,
                    ptr::null_mut(),
                    TimeUnit::zeroed().into_raw(),
                ))
            })?;
        }
        Ok(())
    }

    /// Retrieves information on an embedded sync point.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn get_sync_point_offset(&self, sync_point: &SyncPoint, unit: TimeUnit) -> Result<u32> {
        let mut offset = 0;
        ffi!(FMOD_Sound_GetSyncPointInfo(
            self.as_raw(),
            sync_point.as_raw(),
            ptr::null_mut(),
            0,
            &mut offset,
            unit.into_raw(),
        ))?;
        Ok(offset)
    }

    /// Adds a sync point at a specific time within the sound.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn add_sync_point(
        &self,
        offset: Time,
        name: Option<&CStr8>,
    ) -> Result<Handle<'static, SyncPoint>> {
        let mut point = ptr::null_mut();
        ffi!(FMOD_Sound_AddSyncPoint(
            self.as_raw(),
            offset.value,
            offset.unit.into_raw(),
            name.map(|s| s.as_c_str().as_ptr()).unwrap_or(ptr::null()),
            &mut point,
        ))?;
        Ok(unsafe { Handle::from_raw(point) })
    }

    /// Deletes a sync point within the sound.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    pub fn delete_sync_point(&self, sync_point: Handle<'_, SyncPoint>) -> Result {
        ffi!(FMOD_Sound_DeleteSyncPoint(
            self.as_raw(),
            Handle::into_raw(sync_point),
        ))?;
        Ok(())
    }
}
