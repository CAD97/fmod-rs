use fmod::{raw::*, *};

// We make the potentially dangerous assumption that for the FMOD_CHANNELCONTROL
// API, FMOD_Channel_Op and FMOD_ChannelGroup_Op call the same static function
// that the C++ API exposes as FMOD::ChannelControl::op. This allows us to have
// a deduplicated API surface for the Rust API like exists for the C++ API. It's
// guaranteed that the C pointers and the C++ pointers are interchangeable, so
// this is likely a safe assumption, but it would be more correct to create new
// C ABI functions that take FMOD_CHANNELCONTROL and call into the C++ API.

/// # Playback.
impl ChannelControl {
    /// Retrieves the playing state.
    ///
    /// A Channel is considered playing after [`System::play_sound`] or
    /// [`System::play_dsp`], even if it is paused.
    ///
    /// A [`ChannelGroup`] is considered playing if it has any playing Channels.
    pub fn is_playing(&self) -> Result<bool> {
        let mut isplaying = 0;
        ffi!(FMOD_Channel_IsPlaying(self.as_raw() as _, &mut isplaying))?;
        Ok(isplaying != 0)
    }

    /// Stops the Channel (or all Channels in nested ChannelGroups) from playing.
    ///
    /// This will free up internal resources for reuse by the virtual voice system.
    ///
    /// [`Channel`]s are stopped automatically when their playback position
    /// reaches the length of the [`Sound`] being played. This is not the case
    /// however if the [`Channel`] is playing a DSP or the [`Sound`] is looping,
    /// in which case the [`Channel`] will continue playing until stop is called.
    /// Once stopped, the [`Channel`] handle will become invalid and can be
    /// discarded and any API calls made with it will return
    /// [`Error::InvalidHandle`].
    pub fn stop(&self) -> Result {
        ffi!(FMOD_Channel_Stop(self.as_raw() as _))?;
        Ok(())
    }

    /// Sets the paused state.
    ///
    /// Pause halts playback which effectively freezes [`Channel::get_position`]
    /// and [`ChannelControl::get_dsp_clock`] values.
    ///
    /// An individual pause state is kept for each object, pausing a parent
    /// [`ChannelGroup`] will effectively pause this object however when queried
    /// the individual pause state is returned.
    pub fn set_paused(&self, paused: bool) -> Result {
        let paused = paused as i32;
        ffi!(FMOD_Channel_SetPaused(self.as_raw() as _, paused))?;
        Ok(())
    }

    /// Retrieves the paused state.
    ///
    /// An individual pause state is kept for each object, a parent
    /// [`ChannelGroup`] being paused will effectively pause this object however
    /// when queried the individual pause state is returned.
    pub fn get_paused(&self) -> Result<bool> {
        let mut paused = 0;
        ffi!(FMOD_Channel_GetPaused(self.as_raw() as _, &mut paused))?;
        Ok(paused != 0)
    }

    /// Sets the playback mode that controls how this object behaves.
    ///
    /// Modes supported:
    ///
    /// - [`Mode::LoopOff`]
    /// - [`Mode::LoopNormal`]
    /// - [`Mode::LoopBidi`]
    /// - [`Mode::D2`]
    /// - [`Mode::D3`]
    /// - [`Mode::HeadRelative3d`]
    /// - [`Mode::WorldRelative3d`]
    /// - [`Mode::InverseRolloff3d`]
    /// - [`Mode::LinearRolloff3d`]
    /// - [`Mode::LinearSquareRolloff3d`]
    /// - [`Mode::InverseTaperedRolloff3d`]
    /// - [`Mode::CustomRolloff3d`]
    /// - [`Mode::IgnoreGeometry3d`]
    /// - [`Mode::VirtualPlayFromStart`]
    ///
    /// When changing the loop mode, sounds created with
    /// [`System::create_stream`] or [`Mode::CreateStream`] may have already
    /// been pre-buffered and executed their loop logic ahead of time before
    /// this call was even made. This is dependent on the size of the sound
    /// versus the size of the stream decode buffer (see [`CreateSoundEx`]).
    /// If this happens, you may need to reflush the stream buffer by calling
    /// [`Channel::set_position`]. Note this will usually only happen if you
    /// have sounds or loop points that are smaller than the stream decode
    /// buffer size.
    ///
    /// When changing the loop mode of sounds created with with
    /// [`System::create_sound`] or [`Mode::CreateSample`], if the sound was set
    /// up as [`Mode::LoopOff`], then set to [`Mode::LoopNormal`] with this
    /// function, the sound may click when playing the end of the sound.
    /// This is because the sound needs to be prepared for looping using
    /// [`Sound::set_mode`], by modifying the content of the PCM data
    /// (i.e. data past the end of the actual sample data) to allow the
    /// interpolators to read ahead without clicking. If you use
    /// [`ChannelControl::set_mode`] it will not do this (because different
    /// Channels may have different loop modes for the same sound) and may click
    /// if you try to set it to looping on an unprepared sound. If you want to
    /// change the loop mode at runtime it may be better to load the sound as
    /// looping first (or use [`Sound::set_mode`]), to let it prepare the data
    /// as if it was looping so that it does not click whenever
    /// [`ChannelControl::set_mode`] is used to turn looping on.
    ///
    /// If [`Mode::IgnoreGeometry3d`] or [`Mode::VirtualPlayFromStart`] is not
    /// specified, the flag will be cleared if it was specified previously.
    pub fn set_mode(&self, mode: Mode) -> Result {
        ffi!(FMOD_Channel_SetMode(self.as_raw() as _, mode.into_raw()))?;
        Ok(())
    }

    /// Retrieves the playback mode that controls how this object behaves.
    pub fn get_mode(&self) -> Result<Mode> {
        let mut mode = 0;
        ffi!(FMOD_Channel_GetMode(self.as_raw() as _, &mut mode))?;
        Ok(Mode::from_raw(mode))
    }

    /// Sets the relative pitch / playback rate.
    ///
    /// Scales playback frequency of [`Channel`] object or if issued on a
    /// [`ChannelGroup`] it scales the frequencies of all [`Channels`]
    /// contained in the [`ChannelGroup`].
    ///
    /// A pitch value of 0.5 represents half pitch (one octave down),
    /// 1.0 represents unmodified pitch, and
    /// 2.0 represents double pitch (one octave up).
    ///
    /// An individual pitch value is kept for each object, changing the pitch of
    /// a parent [`ChannelGroup`] will effectively alter the pitch of this
    /// object however when queried the individual pitch value is returned.
    pub fn set_pitch(&self, pitch: f32) -> Result {
        ffi!(FMOD_Channel_SetPitch(self.as_raw() as _, pitch))?;
        Ok(())
    }

    /// Retrieves the relative pitch / playback rate.
    ///
    /// A pitch value of 0.5 represents half pitch (one octave down),
    /// 1.0 represents unmodified pitch, and
    /// 2.0 represents double pitch (one octave up).
    ///
    /// An individual pitch value is kept for each object, a parent
    /// [`ChannelGroup`] pitch will effectively scale the pitch of this object
    /// however when queried the individual pitch value is returned.
    pub fn get_pitch(&self) -> Result<f32> {
        let mut pitch = 0.0;
        ffi!(FMOD_Channel_GetPitch(self.as_raw() as _, &mut pitch))?;
        Ok(pitch)
    }
}
