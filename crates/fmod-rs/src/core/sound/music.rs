use fmod::{raw::*, *};

/// # Music.
impl Sound {
    /// Gets the number of music channels inside a MOD/S3M/XM/IT/MIDI file.
    pub fn get_music_num_channels(&self) -> Result<i32> {
        let mut channels = 0;
        ffi!(FMOD_Sound_GetMusicNumChannels(self.as_raw(), &mut channels))?;
        Ok(channels)
    }

    /// Sets the volume of a MOD/S3M/XM/IT/MIDI music channel volume.
    pub fn set_music_channel_volume(&self, channel: i32, volume: f32) -> Result {
        ffi!(FMOD_Sound_SetMusicChannelVolume(
            self.as_raw(),
            channel,
            volume,
        ))?;
        Ok(())
    }

    /// Retrieves the volume of a MOD/S3M/XM/IT/MIDI music channel volume.
    pub fn get_music_channel_volume(&self, channel: i32) -> Result<f32> {
        let mut volume = 0.0;
        ffi!(FMOD_Sound_GetMusicChannelVolume(
            self.as_raw(),
            channel,
            &mut volume,
        ))?;
        Ok(volume)
    }

    /// Sets the relative speed of MOD/S3M/XM/IT/MIDI music.
    pub fn set_music_speed(&self, speed: f32) -> Result {
        ffi!(FMOD_Sound_SetMusicSpeed(self.as_raw(), speed))?;
        Ok(())
    }

    /// Retrieves the relative speed of MOD/S3M/XM/IT/MIDI music.
    pub fn get_music_speed(&self) -> Result<f32> {
        let mut speed = 0.0;
        ffi!(FMOD_Sound_GetMusicSpeed(self.as_raw(), &mut speed))?;
        Ok(speed)
    }
}
