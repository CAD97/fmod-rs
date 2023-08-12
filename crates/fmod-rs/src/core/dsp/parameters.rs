use {
    fmod::{effect::*, raw::*, *},
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
    pub fn set_parameter<T: ?Sized + DspParamType>(
        &self,
        index: impl DspParam<T>,
        value: impl Borrow<T>,
    ) -> Result {
        T::set_dsp_parameter(self, index.into(), value.borrow())
    }

    // /// Retrieves a DSP parameter by index.
    // pub fn get_parameter<T: DspParamType>(&self, index: impl DspParam<T>) -> Result<T> {
    //     T::get_dsp_parameter(self, index.into())
    // }

    /// Retrieves the string representation of a DSP parameter by index.
    pub fn get_parameter_string<T: ?Sized + DspParamType>(
        &self,
        index: impl DspParam<T>,
        string: &mut String,
    ) -> Result {
        string.clear();
        let mut bytes = [0; FMOD_DSP_GETPARAM_VALUESTR_LENGTH as usize];
        *string += T::get_dsp_parameter_string(self, index.into(), &mut bytes)?;
        Ok(())
    }

    // set_data_parameter, get_data_parameter, get_data_parameter_string
    // get_parameter_info
}
