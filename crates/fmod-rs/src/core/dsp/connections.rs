use {
    fmod::{raw::*, *},
    std::ptr::{self, NonNull},
};

/// # Connections.
impl Dsp {
    /// Adds a DSP unit as an input to this object.
    ///
    /// When a DSP has multiple inputs the signals are automatically mixed
    /// together, sent to the unit's output(s).
    ///
    /// The returned DSP connection will remain valid until the units are
    /// disconnected.
    pub fn add_input(
        &self,
        input: &Dsp,
        kind: DspConnectionType,
    ) -> Result<NonNull<DspConnection>> {
        let mut connection = ptr::null_mut();
        ffi!(FMOD_DSP_AddInput(
            self.as_raw(),
            input.as_raw(),
            &mut connection,
            kind.into_raw(),
        ))?;
        Ok(unsafe { DspConnection::from_raw(connection) }.into())
    }

    /// Retrieves the DSP unit at the specified index in the input list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the input list is correct, avoid this during time sensitive operations.
    ///
    /// The returned DSP connection will remain valid until the units are
    /// disconnected.
    pub fn get_input(&self, index: i32) -> Result<(&Dsp, NonNull<DspConnection>)> {
        let mut dsp = ptr::null_mut();
        let mut connection = ptr::null_mut();
        ffi!(FMOD_DSP_GetInput(
            self.as_raw(),
            index,
            &mut dsp,
            &mut connection,
        ))?;
        Ok((
            unsafe { Dsp::from_raw(dsp) },
            unsafe { DspConnection::from_raw(connection) }.into(),
        ))
    }

    /// Retrieves the DSP unit at the specified index in the output list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the output list is correct, avoid this during time sensitive operations.
    ///
    /// The returned DSP connection will remain valid until the units are
    /// disconnected.
    pub fn get_output(&self, index: i32) -> Result<(&Dsp, NonNull<DspConnection>)> {
        let mut dsp = ptr::null_mut();
        let mut connection = ptr::null_mut();
        ffi!(FMOD_DSP_GetOutput(
            self.as_raw(),
            index,
            &mut dsp,
            &mut connection,
        ))?;
        Ok((
            unsafe { Dsp::from_raw(dsp) },
            unsafe { DspConnection::from_raw(connection) }.into(),
        ))
    }

    /// Retrieves the number of DSP units in the input list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the input list is correct, avoid this during time sensitive operations.
    pub fn get_num_inputs(&self) -> Result<i32> {
        let mut num_inputs = 0;
        ffi!(FMOD_DSP_GetNumInputs(self.as_raw(), &mut num_inputs))?;
        Ok(num_inputs)
    }

    /// Retrieves the number of DSP units in the output list.
    ///
    /// This will flush the DSP queue (which blocks against the mixer) to ensure
    /// the output list is correct, avoid this during time sensitive operations.
    pub fn get_num_outputs(&self) -> Result<i32> {
        let mut num_outputs = 0;
        ffi!(FMOD_DSP_GetNumOutputs(self.as_raw(), &mut num_outputs))?;
        Ok(num_outputs)
    }

    /// Disconnects all inputs and outputs.
    ///
    /// This is a convenience function that is faster than disconnecting all
    /// inputs and outputs individually.
    pub fn disconnect_all(&self) -> Result {
        ffi!(FMOD_DSP_DisconnectAll(
            self.as_raw(),
            /* inputs */ true as FMOD_BOOL,
            /* outputs */ true as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Disconnects all inputs.
    ///
    /// This is a convenience function that is faster than disconnecting all
    /// inputs individually.
    pub unsafe fn disconnect_all_inputs(&self) -> Result {
        ffi!(FMOD_DSP_DisconnectAll(
            self.as_raw(),
            /* inputs */ true as FMOD_BOOL,
            /* outputs */ false as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Disconnects all outputs.
    ///
    /// This is a convenience function that is faster than disconnecting all
    /// outputs individually.
    pub unsafe fn disconnect_all_outputs(&self) -> Result {
        ffi!(FMOD_DSP_DisconnectAll(
            self.as_raw(),
            /* inputs */ false as FMOD_BOOL,
            /* outputs */ true as FMOD_BOOL,
        ))?;
        Ok(())
    }

    /// Disconnect the specified input DSP.
    ///
    /// If `target` had only one output, after this operation that entire sub
    /// graph will no longer be connected to the DSP network.
    pub fn disconnect_from_input(&self, target: &Dsp) -> Result {
        ffi!(FMOD_DSP_DisconnectFrom(
            self.as_raw(),
            target.as_raw(),
            ptr::null_mut(),
        ))?;
        Ok(())
    }

    /// Disconnect the specified output DSP.
    ///
    /// If `self` had only one output, after this operation this entire sub
    /// graph will no longer be connected to the DSP network.
    pub fn disconnect_from_output(&self, target: &Dsp) -> Result {
        ffi!(FMOD_DSP_DisconnectFrom(
            target.as_raw(),
            self.as_raw(),
            ptr::null_mut(),
        ))?;
        Ok(())
    }
}
