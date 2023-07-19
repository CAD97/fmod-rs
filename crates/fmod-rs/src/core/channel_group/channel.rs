use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Channel management.
impl ChannelGroup {
    /// Retrieves the number of Channels that feed into this group.
    pub fn get_num_channels(&self) -> Result<i32> {
        let mut num_channels = 0;
        ffi!(FMOD_ChannelGroup_GetNumChannels(
            self.as_raw(),
            &mut num_channels
        ))?;
        Ok(num_channels)
    }

    /// Retrieves the Channel at the specified index in the list of Channel inputs.
    pub fn get_channel(&self, index: i32) -> Result<&Channel> {
        let mut channel = ptr::null_mut();
        ffi!(FMOD_ChannelGroup_GetChannel(
            self.as_raw(),
            index,
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }
}
