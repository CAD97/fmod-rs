use {
    fmod::utils::fmod_get_string,
    fmod::{raw::*, *},
    std::borrow::Borrow,
};

/// # Parameters.
impl Dsp {
    // get_data_parameter_index

    /// Retrieves the number of parameters exposed by this unit.
    ///
    /// Use this to enumerate all parameters of a DSP unit with
    /// [`Dsp::get_parameter_info`].
    pub fn get_num_parameters(&self) -> Result<i32> {
        let mut num_params = 0;
        ffi!(FMOD_DSP_GetNumParameters(self.as_raw(), &mut num_params))?;
        Ok(num_params)
    }

    /// Sets a DSP parameter by index.
    pub fn set_parameter<P: DspParamMut>(
        &mut self,
        index: P,
        value: impl Borrow<P::Value>,
    ) -> Result {
        unsafe { self.set_parameter_unchecked(index, value.borrow()) }
    }

    /// Sets a DSP parameter by index.
    ///
    /// # Safety
    ///
    /// Calls must be synchronized to not race with [`get_parameter`] or [`get_parameter_string`].
    pub unsafe fn set_parameter_unchecked<T: ?Sized + DspParamValue>(
        &self,
        index: impl Into<i32>,
        value: &T,
    ) -> Result {
        unsafe { T::set_dsp_parameter_unchecked(self, index.into(), value) }
    }

    /// Retrieves a DSP parameter by index.
    pub fn get_parameter<P: DspParam>(&self, index: P) -> Result<P::Value>
    where
        P::Value: Sized,
    {
        unsafe { P::Value::get_dsp_parameter_unchecked(self, index.into()) }
    }

    /// Retrieves the string representation of a DSP parameter by index.
    pub fn get_parameter_string<P: DspParam>(&self, index: P) -> Result<String> {
        let index = index.into();
        let mut s = String::new();
        unsafe {
            fmod_get_string(&mut s, move |buf| {
                P::Value::get_dsp_parameter_string_unchecked(self, index, buf)
            })?;
        }
        Ok(s)
    }

    // get_parameter_info
}
