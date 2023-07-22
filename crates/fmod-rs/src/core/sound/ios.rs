use fmod::{raw::*, *};

fmod_enum! {
    #[cfg_attr(feature = "unstable", doc(cfg(target_os = "ios")))]
    /// Control whether the sound will use a the dedicated hardware decoder or a
    /// software codec.
    ///
    /// Every devices has a single hardware decoder and unlimited software
    /// decoders.
    #[derive(Default)]
    pub enum AudioQueueCodecPolicy: FMOD_AUDIOQUEUE_CODECPOLICY
    where const { self <= FMOD_AUDIOQUEUE_CODECPOLICY_HARDWAREONLY }
    {
        #[default]
        /// Try hardware first, if it's in use or prohibited by audio session,
        /// try software.
        Default      = FMOD_AUDIOQUEUE_CODECPOLICY_DEFAULT,
        /// `kAudioQueueHardwareCodecPolicy_UseSoftwareOnly` ~ try software,
        /// if not available fail.
        SoftwareOnly = FMOD_AUDIOQUEUE_CODECPOLICY_SOFTWAREONLY,
        /// `kAudioQueueHardwareCodecPolicy_UseHardwareOnly` ~ try hardware,
        /// if not available fail.
        HardwareOnly = FMOD_AUDIOQUEUE_CODECPOLICY_HARDWAREONLY,
    }
}
