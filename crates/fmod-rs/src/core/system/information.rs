use {
    fmod::{raw::*, *},
    std::ptr,
};

/// # Information.
impl System {
    /// Retrieves the FMOD version number.
    ///
    /// Compare against `fmod::VERSION` to make sure header and runtime library
    /// versions match.
    pub fn get_version(&self) -> Result<Version> {
        let mut version = 0;
        ffi!(FMOD_System_GetVersion(self.as_raw(), &mut version))?;
        Ok(Version::from_raw(version))
    }

    /// Retrieves an output type specific internal native interface.
    ///
    /// Reinterpret the returned handle based on the selected output type, not
    /// all types return something.
    ///
    /// - [`OutputType::WavWriter`]: Pointer to stdio `FILE` is returned.
    /// - [`OutputType::WavWriterNrt`]: Pointer to stdio `FILE` is returned.
    /// - [`OutputType::Wasapi`]: Pointer to type `IAudioClient` is returned.
    /// - [`OutputType::Alsa`]: Pointer to type `snd_pcm_t` is returned.
    /// - [`OutputType::CoreAudio`]: Handle of type `AudioUnit` is returned.
    /// - [`OutputType::AudioOut`]: Pointer to type `i32` is returned. Handle returned from `sceAudioOutOpen`.
    pub fn get_output_handle(&self) -> Result<*mut ()> {
        let mut output = ptr::null_mut();
        ffi!(FMOD_System_GetOutputHandle(self.as_raw(), &mut output))?;
        Ok(output.cast())
    }

    /// Retrieves the number of currently playing Channels.
    ///
    /// For differences between real and virtual voices see the Virtual Voices
    /// guide for more information.
    pub fn get_channels_playing(&self) -> Result<ChannelUsage> {
        let mut channels = 0;
        let mut real_channels = 0;
        ffi!(FMOD_System_GetChannelsPlaying(
            self.as_raw(),
            &mut channels,
            &mut real_channels,
        ))?;
        Ok(ChannelUsage {
            all: channels,
            real: real_channels,
        })
    }

    /// Retrieves the amount of CPU used for different parts of the Core engine.
    ///
    /// For readability, the percentage values are smoothed to provide a more
    /// stable output.
    pub fn get_cpu_usage(&self) -> Result<CpuUsage> {
        let mut usage = CpuUsage::default();
        ffi!(FMOD_System_GetCPUUsage(self.as_raw(), usage.as_raw_mut()))?;
        Ok(usage)
    }

    /// Retrieves information about file reads.
    pub fn get_file_usage(&self) -> Result<FileUsage> {
        let mut sample_bytes_read = 0;
        let mut stream_bytes_read = 0;
        let mut other_bytes_read = 0;
        ffi!(FMOD_System_GetFileUsage(
            self.as_raw(),
            &mut sample_bytes_read,
            &mut stream_bytes_read,
            &mut other_bytes_read,
        ))?;
        Ok(FileUsage {
            sample_bytes_read,
            stream_bytes_read,
            other_bytes_read,
        })
    }

    // TODO: figure out mix matrix API
    // get_default_mix_matrix

    /// Retrieves the channel count for a given speaker mode.
    pub fn get_speaker_mode_channels(&self, mode: SpeakerMode) -> Result<usize> {
        let mut channels = 0;
        ffi!(FMOD_System_GetSpeakerModeChannels(
            self.as_raw(),
            mode.into_raw(),
            &mut channels,
        ))?;
        Ok(channels as _)
    }
}

/// A number of playing channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelUsage {
    /// Number of playing [Channel]s (both real and virtual).
    pub all: i32,
    /// Number of playing real (non-virtual) [Channel]s.
    pub real: i32,
}

/// Running total information about file reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileUsage {
    /// Total bytes read from file for loading sample data.
    pub sample_bytes_read: i64,
    /// Total bytes read from file for streaming sounds.
    pub stream_bytes_read: i64,
    /// Total bytes read for non-audio data such as FMOD Studio banks.
    pub other_bytes_read: i64,
}
