use {
    fmod::{raw::*, *},
    std::ptr::{self, NonNull},
};

/// # ChannelGroup management.
impl ChannelGroup {
    /// Adds a ChannelGroup as an input to this group.
    ///
    /// Recursively propagates this object's DSP clock to the added group.
    ///
    /// The returned DSP connection will remain valid until the groups are
    /// disconnected.
    pub fn add_group(&self, group: &ChannelGroup) -> Result<Option<NonNull<DspConnection>>> {
        let mut connection = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_AddGroup(
            self.as_raw(),
            group.as_raw(),
            /* propagate_dsp_clock */ true as _,
            &mut connection,
        ))?;
        Ok(unsafe { DspConnection::from_raw_opt(connection) }.map(Into::into))
    }

    /// Adds a ChannelGroup as an input to this group.
    ///
    /// Unlike [`add_group`](Self::add_group), this does not propagate this
    /// object's DSP clock to the added group, meaning that the two groups'
    /// clocks will be independent.
    ///
    /// The returned DSP connection will remain valid until the groups are
    /// disconnected.
    pub fn add_group_unsynchronized(
        &self,
        group: &ChannelGroup,
    ) -> Result<Option<NonNull<DspConnection>>> {
        let mut connection = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_AddGroup(
            self.as_raw(),
            group.as_raw(),
            /* propagate_dsp_clock */ false as _,
            &mut connection,
        ))?;
        Ok(unsafe { DspConnection::from_raw_opt(connection) }.map(Into::into))
    }

    /// Retrieves the number of ChannelGroups that feed into this group.
    pub fn get_num_groups(&self) -> Result<i32> {
        let mut num_groups = 0;
        ffi!(FMOD_ChannelGroup_GetNumGroups(
            self.as_raw(),
            &mut num_groups
        ))?;
        Ok(num_groups)
    }

    /// Retrieves the ChannelGroup at the specified index in the list of group inputs.
    pub fn get_group(&self, index: i32) -> Result<&ChannelGroup> {
        let mut group = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_GetGroup(self.as_raw(), index, &mut group))?;
        Ok(unsafe { ChannelGroup::from_raw(group) })
    }

    /// Retrieves the ChannelGroup this object outputs to.
    pub fn get_parent_group(&self) -> Result<Option<&ChannelGroup>> {
        let mut group = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_GetParentGroup(self.as_raw(), &mut group))?;
        if group.is_null() {
            Ok(None)
        } else {
            Ok(Some(unsafe { ChannelGroup::from_raw(group) }))
        }
    }
}
