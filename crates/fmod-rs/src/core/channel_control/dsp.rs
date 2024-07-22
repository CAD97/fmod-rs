use {
    fmod::{raw::*, *},
    std::ptr,
};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.
// Decompiling the DLL strongly suggests that this is indeed a sound assumption.

/// # DSP chain configuration
impl ChannelControl {
    /// Head of the DSP chain, equivalent of index 0.
    pub const DSP_HEAD: i32 = FMOD_CHANNELCONTROL_DSP_HEAD;
    /// Built in fader DSP.
    pub const DSP_FADER: i32 = FMOD_CHANNELCONTROL_DSP_FADER;
    /// Tail of the DSP chain, equivalent of the number of DSPs minus 1.
    pub const DSP_TAIL: i32 = FMOD_CHANNELCONTROL_DSP_TAIL;

    /// Adds a DSP unit to the specified index in the DSP chain.
    ///
    /// If `dsp` is already added to an existing object it will be removed and
    /// then added to this object. Releasing the DSP unit without first removing
    /// it from the network will error with [`Error::DspInUse`].
    ///
    /// For detailed information on FMOD's DSP network, read the
    /// [DSP Architecture and Usage] white paper.
    ///
    /// [DSP Architecture and Usage]: https://fmod.com/docs/2.02/api/white-papers-dsp-architecture.html
    pub fn add_dsp(&self, index: i32, dsp: &Dsp) -> Result {
        // FIXME: dropping the DSP without removing it from the mixer network is an error
        ffi!(FMOD_Channel_AddDSP(self.as_raw() as _, index, dsp.as_raw()))?;
        Ok(())
    }

    /// Adds a DSP to the end of the DSP chain (at the [tail](ChannelControl::DSP_TAIL)).
    pub fn push_dsp(&self, dsp: &Dsp) -> Result {
        self.add_dsp(Self::DSP_TAIL, dsp)
    }

    /// Removes the specified DSP unit from the DSP chain.
    ///
    /// # Safety
    ///
    /// Dsp references retrieved via `get_dsp` must not outlive their [`Dsp`].
    pub unsafe fn remove_dsp(&self, dsp: &Dsp) -> Result {
        ffi!(FMOD_Channel_RemoveDSP(self.as_raw() as _, dsp.as_raw()))?;
        Ok(())
    }

    /// Retrieves the number of DSP units in the DSP chain.
    pub fn get_num_dsps(&self) -> Result<i32> {
        let mut num_dsps = 0;
        ffi!(FMOD_Channel_GetNumDSPs(self.as_raw() as _, &mut num_dsps))?;
        Ok(num_dsps)
    }

    /// Retrieves the DSP unit at the specified index in the DSP chain.
    pub fn get_dsp(&self, index: i32) -> Result<&Dsp> {
        let mut dsp = ptr::null_mut();
        ffi!(FMOD_Channel_GetDSP(self.as_raw() as _, index, &mut dsp))?;
        Ok(unsafe { Dsp::from_raw(dsp) })
    }

    /// Retrieves the DSP unit at the head of the DSP chain.
    pub fn get_dsp_head(&self) -> Result<&Dsp> {
        self.get_dsp(Self::DSP_HEAD)
    }

    /// Retrieves the DSP unit at the tail of the DSP chain.
    pub fn get_dsp_tail(&self) -> Result<&Dsp> {
        self.get_dsp(Self::DSP_TAIL)
    }

    /// Sets the index in the DSP chain of the specified DSP.
    ///
    /// This will move a [`Dsp`] already in the [DSP chain] to a new offset.
    ///
    /// [DSP chain]: https://fmod.com/docs/2.02/api/glossary.html#dsp-chain
    pub fn set_dsp_index(&self, dsp: &Dsp, index: i32) -> Result {
        ffi!(FMOD_Channel_SetDSPIndex(
            self.as_raw() as _,
            dsp.as_raw(),
            index,
        ))?;
        Ok(())
    }

    /// Retrieves the index of a DSP inside the Channel or ChannelGroup's
    /// DSP chain.
    ///
    /// See [DSP chain].
    ///
    /// [DSP chain]: https://fmod.com/docs/2.02/api/glossary.html#dsp-chain
    pub fn get_dsp_index(&self, dsp: &Dsp) -> Result<i32> {
        let mut index = 0;
        ffi!(FMOD_Channel_GetDSPIndex(
            self.as_raw() as _,
            dsp.as_raw(),
            &mut index,
        ))?;
        Ok(index)
    }
}

impl<C: ?Sized + Resource> Handle<'_, C>
where
    C: AsRef<ChannelControl>,
{
    /// Removes the specified DSP unit from the DSP chain.
    pub fn remove_dsp(&mut self, dsp: &Dsp) -> Result {
        unsafe { self.as_ref().remove_dsp(dsp) }
    }
}

raw! {
    fmod_typedef! {
        /// References to built in DSP positions that reside in a Channel or ChannelGroup DSP chain.
        ///
        /// Before any [Dsp]s have been added by the user, there is only one [Dsp] available for a [Channel] or [ChannelGroup]. This is of type [DspType::Fader]. This handles volume and panning for a [Channel] or [ChannelGroup].
        /// As only 1 [Dsp] exists by default, initially [ChannelControlDspIndex::Head], [ChannelControlDspIndex::Tail] and [ChannelControlDspIndex::Fader] all reference the same DSP.
        pub enum ChannelControlDspIndex: FMOD_CHANNELCONTROL_DSP_INDEX {
            /// Head of the DSP chain, equivalent of index 0.
            Head  = FMOD_CHANNELCONTROL_DSP_HEAD,
            /// Built in fader DSP.
            Fader = FMOD_CHANNELCONTROL_DSP_FADER,
            /// Tail of the DSP chain, equivalent of the number of [Dsp]s minus 1.
            Tail  = FMOD_CHANNELCONTROL_DSP_TAIL,
        }
    }
}
