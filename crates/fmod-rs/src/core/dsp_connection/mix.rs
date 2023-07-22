use fmod::{raw::*, *};

/// # Mix properties.
impl DspConnection {
    /// Sets the connection's volume scale.
    pub fn set_mix(&self, volume: f32) -> Result<()> {
        ffi!(FMOD_DSPConnection_SetMix(self.as_raw(), volume))?;
        Ok(())
    }

    /// Retrieves the connection's volume scale.
    pub fn get_mix(&self) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_DSPConnection_GetMix(self.as_raw(), &mut volume))?;
        Ok(volume)
    }

    // set_mix_matrix, get_mix_matrix
}
