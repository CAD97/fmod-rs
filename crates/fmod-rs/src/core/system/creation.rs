use {
    crate::file::{
        userasynccancel_listen, userasyncread_listen, userclose_listen, useropen_listen,
        userread_listen, userseek_listen, AsyncListenFileSystem,
    },
    fmod::{raw::*, *},
    std::{ffi::CStr, fmt, marker::PhantomData, mem, ptr},
};

/// # Creation and retrieval.
impl System {
    /// Loads a sound into memory, opens it for streaming or sets it up for
    /// callback based sounds.
    ///
    /// [Mode::CreateSample] will try to load and decompress the whole sound
    /// into memory, use [Mode::CreateStream] to open it as a stream and have
    /// it play back in realtime from disk or another medium.
    /// [Mode::CreateCompressedSample] can also be used for certain formats to
    /// play the sound directly in its compressed format from the mixer.
    ///
    /// - To open a file or URL as a stream, so that it decompresses / reads at
    ///   runtime, instead of loading / decompressing into memory all at the
    ///   time of this call, use the [Mode::CreateStream] flag.
    /// - To open a file or URL as a compressed sound effect that is not
    ///   streamed and is not decompressed into memory at load time, use
    ///   [Mode::CreateCompressedSample]. This is supported with MPEG (mp2/mp3),
    ///   ADPCM/FADPCM, XMA, AT9 and FSB Vorbis files only. This is useful for
    ///   those who want realtime compressed soundeffects, but not the overhead
    ///   of disk access.
    /// - To open a sound as 2D, so that it is not affected by 3D processing,
    ///   use the [Mode::D2] flag. 3D sound commands will be ignored on these
    ///   types of sounds.
    /// - To open a sound as 3D, so that it is treated as a 3D sound, use the
    ///   [Mode::D3] flag.
    ///
    /// Note that [Mode::OpenRaw], [Mode::OpenMemory], [Mode::OpenMemoryPoint],
    /// and [Mode::OpenUser] will not work here, as more information is needed.
    /// Use [`create_sound_ex`](Self::create_sound_ex) instead.
    ///
    /// Use [Mode::NonBlocking] to have the sound open or load in the
    /// background. You can use [Sound::get_open_state] to determine if it has
    /// finished loading / opening or not. While it is loading (not ready),
    /// sound functions are not accessible for that sound.
    ///
    /// To account for slow media that might cause buffer underrun (skipping /
    /// stuttering / repeating blocks of audio) with sounds created with
    /// [Mode::CreateStream], use [System::set_stream_buffer_size] to increase
    /// read ahead.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Use of Mode::NonBlocking is currently not supported for Wasm.
    /// </span></div></div>
    pub fn create_sound(&self, name: &CStr8, mode: Mode) -> Result<Handle<'_, Sound>> {
        if mode & (Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw)
            != Mode::default()
        {
            whoops!("System::create_sound called with advanced mode {mode:?}; use create_sound_ex instead");
            yeet!(Error::InvalidParam);
        }

        let mode = Mode::into_raw(mode);
        let exinfo = ptr::null_mut();
        let mut sound = ptr::null_mut();
        ffi!(FMOD_System_CreateSound(
            self.as_raw(),
            name.as_ptr() as _,
            mode,
            exinfo,
            &mut sound,
        ))?;
        Ok(unsafe { Handle::new(sound) })
    }

    /// Loads a sound into memory, opens it for streaming or sets it up for
    /// callback based sounds.
    ///
    /// Unlike [`create_sound`](Self::create_sound), this function allows you to
    /// specify additional configuration using the [`CreateSoundEx`] type.
    ///
    /// Note that [Mode::OpenRaw], [Mode::OpenMemory], [Mode::OpenMemoryPoint],
    /// and [Mode::OpenUser] will not work without required additional
    /// information provided in the [`CreateSoundEx`] type. If you aren't using
    /// those modes, use [`create_sound`](Self::create_sound) instead.
    ///
    /// Use [Mode::NonBlocking] to have the sound open or load in the
    /// background. You can use [Sound::get_open_state] to determine if it has
    /// finished loading / opening or not. While it is loading (not ready),
    /// sound functions are not accessible for that sound. Do not free memory
    /// provided with [Mode::OpenMemory] if the sound is not in a ready state,
    /// as it will most likely lead to UB and a crash.
    ///
    /// Specifying [`Mode::OpenMemoryPoint`] will POINT to your memory rather
    /// allocating its own sound buffers and duplicating it internally, this
    /// means you cannot free the memory while FMOD is using it, until after
    /// [`Sound::release`] is called.
    ///
    /// With [`Mode::OpenMemoryPoint`], only PCM formats and compressed formats
    /// using [`Mode::CreateCompressedSample`] are supported.
    ///
    /// <div class="item-info"><div class="stab" style="white-space:normal;font-size:inherit">
    /// <span class="emoji">⚠️</span><span>
    /// Use of Mode::NonBlocking is currently not supported for Wasm.
    /// </span></div></div>
    ///
    /// # Safety
    ///
    /// Configuration via `CreateSoundEx` must be correct, and `name_or_data`
    /// must be a valid pointer that lives sufficiently long for the specified
    /// sound creation mode.
    pub unsafe fn create_sound_ex(
        &self,
        name_or_data: *const u8,
        mode: Mode,
        info: CreateSoundEx<'_>,
    ) -> Result<Handle<'_, Sound>> {
        if mode & (Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw)
            == Mode::default()
        {
            whoops!(
                "System::create_sound called with standard mode {mode:?}; use create_sound instead"
            );
            yeet!(Error::InvalidParam);
        }

        let mut sound = ptr::null_mut();
        ffi!(FMOD_System_CreateSound(
            self.as_raw(),
            name_or_data.cast(),
            mode.into_raw(),
            info.as_raw(),
            &mut sound,
        ))?;
        Ok(Handle::new(sound))
    }

    /// Opens a sound for streaming.
    ///
    /// This is a convenience function for [`System::create_sound`] with the
    /// [`Mode::CreateStream`] flag added.
    ///
    /// A stream only has one decode buffer and file handle, and therefore can
    /// only be played once. It cannot play multiple times at once because it
    /// cannot share a stream buffer if the stream is playing at different
    /// positions. Open multiple streams to have them play concurrently.
    ///
    /// If you need access to the extended options, use
    /// [`System::create_sound_ex`] instead and set [`Mode::CreateStream`].
    pub fn create_stream(&self, name: &CStr8, mode: Mode) -> Result<Handle<'_, Sound>> {
        if matches!(
            mode,
            Mode::OpenUser | Mode::OpenMemory | Mode::OpenMemoryPoint | Mode::OpenRaw
        ) {
            whoops!("System::create_stream called with extended mode {mode:?}; use create_sound_ex instead");
            yeet!(Error::InvalidParam);
        }

        let mode = Mode::into_raw(mode);
        let exinfo = ptr::null_mut();
        let mut sound = ptr::null_mut();
        ffi!(FMOD_System_CreateStream(
            self.as_raw(),
            name.as_ptr() as _,
            mode,
            exinfo,
            &mut sound,
        ))?;
        Ok(unsafe { Handle::new(sound) })
    }

    // TODO: pub fn create_dsp

    /// Create a DSP object given a built in type index.
    ///
    /// A DSP object is a module that can be inserted into the mixing graph to
    /// allow sound filtering or sound generation. See the
    /// [DSP architecture guide] for more information.
    ///
    /// [DSP architecture guide]: file:///C:/Program%20Files%20(x86)/FMOD%20SoundSystem/FMOD%20Studio%20API%20Windows/doc/FMOD%20API%20User%20Manual/white-papers-dsp-architecture.html
    ///
    /// DSPs must be attached to the DSP graph before they become active, either
    /// via [ChannelControl::add_dsp] or [Dsp::add_input].
    ///
    /// Using [DspType::VstPlugin] or [DspType::WinampPlugin] will return the
    /// first loaded plugin of this type. To access other plugins of these
    /// types, use [System::create_dsp_by_plugin] instead.
    pub fn create_dsp_by_type(&self, kind: DspType) -> Result<Handle<'_, Dsp>> {
        let mut dsp = ptr::null_mut();
        ffi!(FMOD_System_CreateDSPByType(
            self.as_raw(),
            kind.into_raw(),
            &mut dsp,
        ))?;
        Ok(unsafe { Handle::new(dsp) })
    }

    /// Create a ChannelGroup object.
    ///
    /// [ChannelGroup]s can be used to assign / group [Channel]s, for things
    /// such as volume scaling. [ChannelGroup]s are also used for sub-mixing.
    /// Any [Channel]s that are assigned to a [ChannelGroup] get submixed into
    /// that [ChannelGroup]'s 'tail' [Dsp]. See [ChannelControl::DSP_TAIL].
    ///
    /// If a [ChannelGroup] has an effect added to it, the effect is processed
    /// post-mix from the [Channel]s and [ChannelGroup]s below it in the mix
    /// hierarchy. See the [DSP architecture guide] for more information.
    ///
    /// [DSP architecture guide]: file:///C:/Program%20Files%20(x86)/FMOD%20SoundSystem/FMOD%20Studio%20API%20Windows/doc/FMOD%20API%20User%20Manual/white-papers-dsp-architecture.html
    ///
    /// All [ChannelGroup]s will initially output directly to the master
    /// [ChannelGroup] (See [System::get_master_channel_group]).[ChannelGroup]s
    /// can be re-parented this with [ChannelGroup::add_group].
    pub fn create_channel_group(&self, name: &CStr8) -> Result<Handle<'_, ChannelGroup>> {
        let mut channel_group = ptr::null_mut();
        ffi!(FMOD_System_CreateChannelGroup(
            self.as_raw(),
            name.as_ptr() as _,
            &mut channel_group,
        ))?;
        Ok(unsafe { Handle::new(channel_group) })
    }

    /// Creates a SoundGroup object.
    ///
    /// A [SoundGroup] is a way to address multiple [Sound]s at once with group
    /// level commands, such as:
    /// - Attributes of [Sound]s that are playing or about to be played, such as
    ///   volume. See [SoundGroup::set_volume].
    /// - Control of playback, such as stopping [Sound]s. See
    ///   [SoundGroup::stop].
    /// - Playback behavior such as 'max audible', to limit playback of certain
    ///   types of Sounds. See [SoundGroup::set_max_audible].
    pub fn create_sound_group(&self, name: &CStr8) -> Result<Handle<'_, SoundGroup>> {
        let mut sound_group = ptr::null_mut();
        ffi!(FMOD_System_CreateSoundGroup(
            self.as_raw(),
            name.as_ptr() as _,
            &mut sound_group,
        ))?;
        Ok(unsafe { Handle::new(sound_group) })
    }

    /// Creates a 'virtual reverb' object. This object reacts to 3D location and
    /// morphs the reverb environment based on how close it is to the reverb
    /// object's center.
    ///
    /// Multiple reverb objects can be created to achieve a multi-reverb
    /// environment. 1 Physical reverb object is used for all 3D reverb objects
    /// (slot 0 by default).
    ///
    /// The 3D reverb object is a sphere having 3D attributes (position, minimum
    /// distance, maximum distance) and reverb properties.
    ///
    /// The properties and 3D attributes of all reverb objects collectively
    /// determine, along with the listener's position, the settings of and
    /// input gains into a single 3D reverb [Dsp].
    ///
    /// When the listener is within the sphere of effect of one or more 3D
    /// reverbs, the listener's 3D reverb properties are a weighted combination
    /// of such 3D reverbs.
    ///
    /// When the listener is outside all of the reverbs, no reverb is applied.
    ///
    /// [System::set_reverb_properties] can be used to create an alternative
    /// reverb that can be used for 2D and background global reverb.
    ///
    /// To avoid this reverb interfering with the reverb slot used by the 3D
    /// reverb, 2D reverb should use a different slot id with
    /// [System::set_reverb_properties], otherwise
    /// [AdvancedSettings::reverb_3d_instance] can also be used to place 3D
    /// reverb on a different physical reverb slot.
    ///
    /// Use [ChannelControl::set_reverb_properties] to turn off reverb for 2D
    /// sounds (ie set wet = 0).
    ///
    /// Creating multiple reverb objects does not impact performance. These are
    /// 'virtual reverbs'. There will still be only one physical reverb [Dsp]
    /// running that just morphs between the different virtual reverbs.
    ///
    /// Note about physical reverb [Dsp] unit allocation. To remove the [Dsp]
    /// unit and the associated CPU cost, first make sure all 3D reverb objects
    /// are released. Then call [System::set_reverb_properties] with the 3D
    /// reverb's slot ID (default is 0) with a property point of 0 or NULL, to
    /// signal that the physical reverb instance should be deleted.
    ///
    /// If a 3D reverb is still present, and [System::set_reverb_properties]
    /// function is called to free the physical reverb, the 3D reverb system
    /// will immediately recreate it upon the next [System::update] call.
    ///
    /// Note that the 3D reverb system will not affect Studio events unless it
    /// is explicitly enabled by calling
    /// [studio::EventInstance::set_reverb_level] on each event instance.
    pub fn create_reverb_3d(&self) -> Result<Handle<'_, Reverb3d>> {
        let mut reverb = ptr::null_mut();
        ffi!(FMOD_System_CreateReverb3D(self.as_raw(), &mut reverb))?;
        Ok(unsafe { Handle::new(reverb) })
    }

    /// Creates a Channel to play a Sound. The channel starts paused.
    ///
    /// When a sound is played, it will use the sound's default frequency and
    /// priority. See [Sound::set_defaults].
    ///
    /// A sound defined as [Mode::D3] will by default play at the 3D position of
    /// the listener. To set the 3D position of the [Channel] before the sound
    /// is audible, start the [Channel] paused by setting the `paused` parameter
    /// to true, and call [ChannelControl::set_3D_attributes].
    ///
    /// Specifying a `channel_group` as part of `play_sound` is more efficient
    /// than using [Channel::set_channel_group] after play_sound, and could
    /// avoid audible glitches if the play_sound is not in a paused state.
    ///
    /// [Channel]s are reference counted to handle dead or stolen [Channel]
    /// handles. See the white paper on [Channel handles] for more information.
    ///
    /// Playing more [Sound]s than physical [Channel]s allow is handled with
    /// virtual voices. See the white paper on [Virtual Voices] for more
    /// information.
    ///
    /// [Channel handles]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-handle-system.html#core-api-channels
    /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
    pub fn create_sound_channel(
        &self,
        sound: &Sound,
        channel_group: Option<&ChannelGroup>,
    ) -> Result<&Channel> {
        let sound = Sound::as_raw(sound);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_PlaySound(
            self.as_raw(),
            sound,
            channelgroup,
            true as FMOD_BOOL, // paused
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    /// Plays a Sound on a Channel. The channel is created unpaused.
    ///
    /// See [`System::create_sound_channel`] for more information. Use that
    /// method to start a channel paused and allow altering attributes without
    /// the channel being audible, then follow it up with a call to
    /// [`ChannelControl::set_paused`] with `paused` = false.
    pub fn play_sound(
        &self,
        sound: &Sound,
        channel_group: Option<&ChannelGroup>,
    ) -> Result<&Channel> {
        let sound = Sound::as_raw(sound);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_PlaySound(
            self.as_raw(),
            sound,
            channelgroup,
            false as FMOD_BOOL, // paused
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    /// Creates a channel to plays a DSP along with any of its inputs. The
    /// channel starts paused.
    ///
    /// Specifying a `channel_group` as part of play_dsp is more efficient
    /// than using `Channel::set_channel_group` after play_dsp, and could avoid
    /// audible glitches if the play_dsp is not in a paused state.
    ///
    /// [Channel]s are reference counted to handle dead or stolen [Channel]
    /// handles. See the white paper on [Channel handles] for more information.
    ///
    /// Playing more [Sound]s or [Dsp]s than physical [Channel]s allow is
    /// handled with virtual voices. See the white paper on [Virtual Voices]
    /// for more information.
    ///
    /// [Channel handles]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-handle-system.html#core-api-channels
    /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
    pub fn create_dsp_channel(
        &self,
        dsp: &Dsp,
        channel_group: Option<&ChannelGroup>,
    ) -> Result<&Channel> {
        let dsp = Dsp::as_raw(dsp);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_PlayDSP(
            self.as_raw(),
            dsp,
            channelgroup,
            true as FMOD_BOOL, // paused
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    /// Plays a DSP along with any of its inputs on a Channel. The channel is
    /// created unpaused.
    ///
    /// See [`System::create_dsp_channel`] for more information. Use that method
    /// to start a channel paused and allow altering attributes without the
    /// channel being audible, then follow it up with a call to
    /// [`ChannelControl::set_paused`] with `paused` = false.
    pub fn play_dsp(&self, dsp: &Dsp, channel_group: Option<&ChannelGroup>) -> Result<&Channel> {
        let dsp = Dsp::as_raw(dsp);
        let channelgroup = channel_group
            .map(ChannelGroup::as_raw)
            .unwrap_or(ptr::null_mut());
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_PlayDSP(
            self.as_raw(),
            dsp,
            channelgroup,
            false as FMOD_BOOL, // paused
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    /// Retrieves a handle to a Channel by ID.
    ///
    /// This function is mainly for getting handles to existing (playing)
    /// [Channel]s and setting their attributes. The only way to 'create' an
    /// instance of a [Channel] for playback is to use [System::play_sound] or
    /// [System::play_dsp].
    pub fn get_channel(&self, channel_id: i32) -> Result<&Channel> {
        let mut channel = ptr::null_mut();
        ffi!(FMOD_System_GetChannel(
            self.as_raw(),
            channel_id,
            &mut channel,
        ))?;
        Ok(unsafe { Channel::from_raw(channel) })
    }

    // TODO: pub fn get_dsp_info_by_type

    /// Retrieves the master ChannelGroup that all sounds ultimately route to.
    ///
    /// This is the default [ChannelGroup] that [Channel]s play on, unless a
    /// different [ChannelGroup] is specified with [System::play_sound],
    /// [System::play_dsp], or [Channel::set_channel_group].  
    /// A master [ChannelGroup] can be used to do things like set the 'master
    /// volume' for all playing [Channel]s. See [ChannelControl::set_volume].
    pub fn get_master_channel_group(&self) -> Result<&ChannelGroup> {
        let mut channelgroup = ptr::null_mut();
        ffi!(FMOD_System_GetMasterChannelGroup(
            self.as_raw(),
            &mut channelgroup,
        ))?;
        Ok(unsafe { ChannelGroup::from_raw(channelgroup) })
    }

    /// Retrieves the default SoundGroup, where all sounds are placed when they
    /// are created.
    ///
    /// If [SoundGroup] is released, the [Sound]s will be put back into this
    /// [SoundGroup].
    pub fn get_master_sound_group(&self) -> Result<&SoundGroup> {
        let mut soundgroup = ptr::null_mut();
        ffi!(FMOD_System_GetMasterSoundGroup(
            self.as_raw(),
            &mut soundgroup,
        ))?;
        Ok(unsafe { SoundGroup::from_raw(soundgroup) })
    }
}

/// Additional options for creating a [`Sound`].
///
/// Loading a file from memory:
/// - Create the sound using the [`Mode::OpenMemory`] flag.
/// - Specify `length` for the size of the memory block in bytes.
///
/// Loading a file from within another larger (possibly wad/pak) file,
/// by giving the loader an offset and length:
/// - Specify `file_offset` and `length`.
///
/// Create a user created / non-file based sound:
/// - Create the sound using the [`Mode::OpenUser`] flag.
/// - Specify `default_frequency`, `num_channels` and `format`.
///
/// Load an FSB stream seeking to a specific subsound in one file operation:
/// - Create the sound using the [`Mode::CreateStream`] flag.
/// - Specify `initial_subsound`.
///
/// Load a subset of the Sounds in an FSB saving memory:
/// - Specify `inclusion_list`.
/// - Optionally set `num_subsounds`, saves memory and causes
///   [`Sound::get_sub_sound`] to index into `inclusion_list`.
///
/// Capture sound data as it is decoded:
/// - Specify `pcm_callback`.
///
/// Provide a custom DLS for MIDI playback:
/// - Specify `dls_name`.
///
/// Setting the `decode_buffer_size` is for CPU intensive codecs that may be
/// causing stuttering, not file intensive codecs (i.e. those from CD or net
/// streams) which are normally altered with [`System::set_stream_buffer_size`].
/// As an example of CPU intensive codecs, an MP3 file will take more CPU to
/// decode than a PCM wav file.
///
/// If you have a stuttering effect, then it is using more CPU than the decode
/// buffer playback rate can keep up with. Increasing the `decode_buffer_size`
/// will most likely solve this problem.
///
/// FSB codec. If `inclusion_list` and `num_subsounds` are used together, this
/// will trigger a special mode where subsounds are shuffled down to save memory
/// (useful for large FSB files where you only want to load 1 sound). There will
/// be no gaps, ie no null subsounds. As an example, if there are 10,000
/// subsounds and there is an `inclusion_list` with only 1 entry, and
/// `num_subsounds == 1`, then subsound 0 will be that entry, and there will
/// only be the memory allocated for 1 subsound. Previously there would still be
/// 10,000 subsound pointers and other associated codec entries allocated along
/// with it multiplied by 10,000.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CreateSoundEx<'a> {
    info: FMOD_CREATESOUNDEXINFO,
    marker: PhantomData<&'a u8>,
}

impl Default for CreateSoundEx<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateSoundEx<'_> {
    /// Create a new `CreateSoundEx` with default values.
    pub fn new() -> Self {
        let mut info: FMOD_CREATESOUNDEXINFO = unsafe { std::mem::zeroed() };
        info.cbsize = mem::size_of::<FMOD_CREATESOUNDEXINFO>() as i32;
        Self {
            info,
            marker: PhantomData,
        }
    }

    fn as_raw(&self) -> *mut FMOD_CREATESOUNDEXINFO {
        &self.info as *const _ as *mut _
    }
}

impl<'a> CreateSoundEx<'a> {
    /// Bytes to read starting at `file_offset`, or length of `Sound` to create
    /// for [`Mode::OpenUser`], or length of `name_or_data` for
    /// [`Mode::OpenMemory`] / [`Mode::OpenMemoryPoint`].
    pub fn length(mut self, length: u32) -> Self {
        self.info.length = length;
        self
    }

    /// File offset to start reading from.
    pub fn file_offset(mut self, offset: u32) -> Self {
        self.info.fileoffset = offset;
        self
    }

    /// Number of channels in sound data for [`Mode::OpenUser`] /
    /// [`Mode::OpenRaw`].
    pub fn num_channels(mut self, channels: i32) -> Self {
        self.info.numchannels = channels;
        self
    }

    /// Default frequency of sound data for [`Mode::OpenUser`] /
    /// [`Mode::OpenRaw`].
    pub fn default_frequency(mut self, frequency: i32) -> Self {
        self.info.defaultfrequency = frequency;
        self
    }

    /// Format of sound data for [`Mode::OpenUser`] / [`Mode::OpenRaw`].
    pub fn format(mut self, format: SoundFormat) -> Self {
        self.info.format = format.into_raw();
        self
    }

    /// Size of the decoded buffer for [`Mode::CreateStream`], or the block size
    /// used with [`pcm_callback`] for [`Mode::OpenUser`].
    pub fn decode_buffer_size(mut self, size: u32) -> Self {
        self.info.decodebuffersize = size;
        self
    }

    /// Initial subsound to seek to for [`Mode::CreateStream`].
    pub fn initial_subsound(mut self, subsound: i32) -> Self {
        self.info.initialsubsound = subsound;
        self
    }

    /// Number of subsounds available for [`Mode::OpenUser`], or maximum
    /// subsounds to load from file.
    pub fn num_subsounds(mut self, subsounds: i32) -> Self {
        self.info.numsubsounds = subsounds;
        self
    }

    /// List of subsound indices to load from file.
    pub fn inclusion_list(mut self, list: &'a [i32]) -> Self {
        self.info.inclusionlist = list.as_ptr() as *mut i32;
        self.info.inclusionlistnum = list.len() as i32;
        self
    }

    /// Callbacks to provide audio and seek data for [`Mode::OpenUser`], or
    /// capture audio as it is decoded.
    pub fn pcm_callback<F: PcmCallback>(mut self) -> Self {
        self.info.pcmreadcallback = Some(pcm_read_callback::<F>);
        self.info.pcmsetposcallback = Some(pcm_setpos_callback::<F>);
        self
    }

    /// Callback to notify completion for [`Mode::Nonblocking`], occurs during
    /// creation and seeking / restarting streams.
    pub fn nonblock_callback<F: NonBlockCallback>(mut self) -> Self {
        self.info.nonblockcallback = Some(non_block_callback::<F>);
        self
    }

    /// File path for a [`SoundType::Dls`] sample set to use when loading a
    /// [`SoundType::Midi`] file, see type-level documentation for defaults.
    pub fn dls_name(mut self, name: &'a CStr) -> Self {
        self.info.dlsname = name.as_ptr();
        self
    }

    /// Key for encrypted [`SoundType::Fsb`] file, cannot be used in conjunction
    /// with [`Mode::OpenMemoryPoint`].
    pub fn encryption_key(mut self, key: &'a CStr) -> Self {
        self.info.encryptionkey = key.as_ptr();
        self
    }

    /// Maximum voice count for [`SoundType::Midi`] / [`SoundType::It`].
    pub fn max_polyphony(mut self, polyphony: i32) -> Self {
        self.info.maxpolyphony = polyphony;
        self
    }

    /// Attempt to load using the specified type first instead of loading in
    /// codec priority order.
    pub fn suggested_sound_type(mut self, sound_type: SoundType) -> Self {
        self.info.suggestedsoundtype = sound_type.into_raw();
        self
    }

    /// Callbacks for file operations.
    pub fn file_system<FS: AsyncListenFileSystem>(mut self) -> Self {
        self.info.fileuseropen = Some(useropen_listen::<FS>);
        self.info.fileuserclose = Some(userclose_listen::<FS>);
        self.info.fileuserread = Some(userread_listen::<FS>);
        self.info.fileuserseek = Some(userseek_listen::<FS>);
        self.info.fileuserasyncread = Some(userasyncread_listen::<FS>);
        self.info.fileuserasynccancel = Some(userasynccancel_listen::<FS>);
        self
    }

    /// Buffer size for reading the file, -1 to disable buffering.
    pub fn file_buffer_size(mut self, size: i32) -> Self {
        self.info.filebuffersize = size;
        self
    }

    /// Custom ordering of speakers for this sound data.
    pub fn channel_order(mut self, order: ChannelOrder) -> Self {
        self.info.channelorder = order.into_raw();
        self
    }

    /// SoundGroup to place this Sound in once created.
    pub fn initial_sound_group(mut self, group: &'a SoundGroup) -> Self {
        self.info.initialsoundgroup = group.as_raw();
        self
    }

    /// Initial position to seek to for [`Mode::CreateStream`].
    pub fn initial_seek_position(mut self, position: Time) -> Self {
        self.info.initialseekposition = position.value;
        self.info.initialseekpostype = position.unit.into_raw();
        self
    }

    /// Ignore file callbacks from [`System::set_file_system`] and this ex info.
    pub fn ignore_set_filesystem(mut self) -> Self {
        self.info.ignoresetfilesystem = 1;
        self
    }

    /// Hardware / software decoding policy for [`SoundType::AudioQueue`].
    pub fn audio_queue_policy(mut self, policy: AudioQueueCodecPolicy) -> Self {
        self.info.audioqueuepolicy = policy.into_raw() as u32;
        self
    }

    /// Mixer granularity for [SoundType::Midi] sounds, smaller numbers give a
    /// more accurate reproduction at the cost of higher CPU usage.
    pub fn min_midi_granularity(mut self, granularity: u32) -> Self {
        self.info.minmidigranularity = granularity;
        self
    }

    /// Thread index to execute [Mode::Nonblocking] loads on for parallel
    /// `Sound` loading.
    pub fn non_block_tread_id(mut self, thread_id: i32) -> Self {
        self.info.nonblockthreadid = thread_id;
        self
    }

    /// GUID of already loaded [`SoundType::Fsb`] file to reduce disk access.
    pub fn fsb_guid(mut self, guid: &'a Guid) -> Self {
        self.info.fsbguid = guid.as_raw() as *const FMOD_GUID as *mut FMOD_GUID;
        self
    }
}

impl fmt::Debug for CreateSoundEx<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("CreateSoundEx");
        macro_rules! d {
            ($raw:ident, $rust:ident) => {
                if self.info.$raw != unsafe { std::mem::zeroed() } {
                    d.field(stringify!($rust), &self.info.$raw);
                }
            };
        }
        d!(length, length);
        d!(fileoffset, file_offset);
        d!(numchannels, num_channels);
        d!(defaultfrequency, default_frequency);
        d!(format, format);
        d!(decodebuffersize, decode_buffer_size);
        d!(initialsubsound, initial_sub_sound);
        d!(numsubsounds, num_sub_sounds);
        d!(maxpolyphony, max_polyphony);
        d!(suggestedsoundtype, suggested_sound_type);
        d!(filebuffersize, file_buffer_size);
        d!(channelorder, channel_order);
        d!(initialseekposition, initial_seek_position);
        d!(initialseekpostype, initial_seek_position_type);
        d!(ignoresetfilesystem, ignore_set_filesystem);
        d!(audioqueuepolicy, audio_queue_policy);
        d!(minmidigranularity, min_midi_granularity);
        d!(nonblockthreadid, non_block_tread_id);
        d.finish_non_exhaustive()
    }
}
