use fmod::{raw::*, *};

opaque!(class ChannelGroup = FMOD_CHANNELGROUP, FMOD_ChannelGroup_*);

/// General control functionality.
impl ChannelGroup {
    // snip

    pub fn set_paused(&self, paused: bool) -> Result {
        let paused = paused as i32;
        fmod_try!(FMOD_ChannelGroup_SetPaused(self.as_raw(), paused));
        Ok(())
    }

    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        fmod_try!(FMOD_ChannelGroup_GetPaused(self.as_raw(), &mut paused));
        Ok(paused != 0)
    }

    // snip

    pub fn set_pitch(&self, pitch: f32) -> Result {
        fmod_try!(FMOD_ChannelGroup_SetPitch(self.as_raw(), pitch));
        Ok(())
    }

    pub fn get_pitch(&self) -> Result<f32> {
        let mut pitch = 0.0;
        fmod_try!(FMOD_ChannelGroup_GetPitch(self.as_raw(), &mut pitch));
        Ok(pitch)
    }

    // snip

    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        fmod_try!(FMOD_ChannelGroup_IsPlaying(self.as_raw(), &mut isplaying));
        Ok(isplaying != 0)
    }
}
