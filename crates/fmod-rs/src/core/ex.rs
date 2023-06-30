use {
    crate::file::{
        userasynccancel_listen, userasyncread_listen, userclose_listen, useropen_listen,
        userread_listen, userseek_listen, AsyncListenFileSystem,
    },
    fmod::{raw::*, *},
    std::{
        ffi::{c_void, CStr},
        fmt,
        marker::PhantomData,
        mem, ptr, slice,
    },
};

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
#[derive(Clone, Copy)]
pub struct CreateSoundEx<'a> {
    info: FMOD_CREATESOUNDEXINFO,
    marker: PhantomData<&'a u8>,
}

pub trait PcmCallback {
    fn read(sound: &Sound, data: &mut [u8]) -> Result;
    fn seek(sound: &Sound, subsound: i32, position: u32, pos_type: TimeUnit) -> Result;
}

pub trait NonblockCallback {
    fn notify(sound: &Sound, result: Result) -> Result;
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

    /// Dispatch the sound creation.
    ///
    /// # Safety
    ///
    /// Information provided by the builder methods must agree with the sound
    /// mode provided, and `name_or_data` must be valid as expected for creating
    /// the sound in the selected mode.
    ///
    /// Consult the type-level documentation as well as [`System::create_sound`]
    /// and [`create_sound_ex`](System::create_sound_ex) for more information.
    pub unsafe fn create<'a>(
        &self,
        system: &'a System,
        name_or_data: *const u8,
        mode: Mode,
    ) -> Result<Handle<'a, Sound>> {
        let mut sound = ptr::null_mut();
        ffi!(FMOD_System_CreateSound(
            system.as_raw(),
            name_or_data.cast(),
            mode.into_raw(),
            &self.info as *const _ as *mut _,
            &mut sound,
        ))?;
        Ok(Handle::new(sound))
    }
}

impl<'a> CreateSoundEx<'a> {
    /// Bytes to read starting at `file_offset`, or length of `Sound` to create
    /// for [`Mode::OpenUser`], or length of `name_or_data` for
    /// [`Mode::OpenMemory`] / [`Mode::OpenMemoryPoint`].
    pub fn length(&mut self, length: u32) -> &mut Self {
        self.info.length = length;
        self
    }

    /// File offset to start reading from.
    pub fn file_offset(&mut self, offset: u32) -> &mut Self {
        self.info.fileoffset = offset;
        self
    }

    /// Number of channels in sound data for [`Mode::OpenUser`] /
    /// [`Mode::OpenRaw`].
    pub fn num_channels(&mut self, channels: i32) -> &mut Self {
        self.info.numchannels = channels;
        self
    }

    /// Default frequency of sound data for [`Mode::OpenUser`] /
    /// [`Mode::OpenRaw`].
    pub fn default_frequency(&mut self, frequency: i32) -> &mut Self {
        self.info.defaultfrequency = frequency;
        self
    }

    /// Format of sound data for [`Mode::OpenUser`] / [`Mode::OpenRaw`].
    pub fn format(&mut self, format: SoundFormat) -> &mut Self {
        self.info.format = format.into_raw();
        self
    }

    /// Size of the decoded buffer for [`Mode::CreateStream`], or the block size
    /// used with [`pcm_callback`] for [`Mode::OpenUser`].
    pub fn decode_buffer_size(&mut self, size: u32) -> &mut Self {
        self.info.decodebuffersize = size;
        self
    }

    /// Initial subsound to seek to for [`Mode::CreateStream`].
    pub fn initial_subsound(&mut self, subsound: i32) -> &mut Self {
        self.info.initialsubsound = subsound;
        self
    }

    /// Number of subsounds available for [`Mode::OpenUser`], or maximum
    /// subsounds to load from file.
    pub fn num_subsounds(&mut self, subsounds: i32) -> &mut Self {
        self.info.numsubsounds = subsounds;
        self
    }

    /// List of subsound indices to load from file.
    pub fn inclusion_list(&mut self, list: &'a [i32]) -> &mut Self {
        self.info.inclusionlist = list.as_ptr() as *mut i32;
        self.info.inclusionlistnum = list.len() as i32;
        self
    }

    /// Callbacks to provide audio and seek data for [`Mode::OpenUser`], or
    /// capture audio as it is decoded.
    pub fn pcm_callback<F: PcmCallback>(&mut self) -> &mut Self {
        unsafe extern "system" fn pcm_read_callback<F: PcmCallback>(
            sound: *mut FMOD_SOUND,
            data: *mut c_void,
            datalen: u32,
        ) -> FMOD_RESULT {
            let sound = Sound::from_raw(sound);
            let data = slice::from_raw_parts_mut(data as *mut u8, datalen as usize);
            match F::read(sound, data) {
                Ok(()) => FMOD_OK,
                Err(e) => e.into_raw(),
            }
        }

        unsafe extern "system" fn pcm_setpos_callback<F: PcmCallback>(
            sound: *mut FMOD_SOUND,
            subsound: i32,
            position: u32,
            postype: FMOD_TIMEUNIT,
        ) -> FMOD_RESULT {
            let sound = Sound::from_raw(sound);
            match F::seek(sound, subsound, position, TimeUnit::from_raw(postype)) {
                Ok(()) => FMOD_OK,
                Err(e) => e.into_raw(),
            }
        }

        self.info.pcmreadcallback = Some(pcm_read_callback::<F>);
        self.info.pcmsetposcallback = Some(pcm_setpos_callback::<F>);
        self
    }

    /// Callback to notify completion for [`Mode::Nonblocking`], occurs during
    /// creation and seeking / restarting streams.
    pub fn nonblock_callback<F: NonblockCallback>(&mut self) -> &mut Self {
        unsafe extern "system" fn nonblock_callback<F: NonblockCallback>(
            sound: *mut FMOD_SOUND,
            result: FMOD_RESULT,
        ) -> FMOD_RESULT {
            let sound = Sound::from_raw(sound);
            match F::notify(sound, Error::from_raw(result)) {
                Ok(()) => FMOD_OK,
                Err(e) => e.into_raw(),
            }
        }

        self.info.nonblockcallback = Some(nonblock_callback::<F>);
        self
    }

    /// File path for a [`SoundType::Dls`] sample set to use when loading a
    /// [`SoundType::Midi`] file, see type-level documentation for defaults.
    pub fn dls_name(&mut self, name: &'a CStr) -> &mut Self {
        self.info.dlsname = name.as_ptr();
        self
    }

    /// Key for encrypted [`SoundType::Fsb`] file, cannot be used in conjunction
    /// with [`Mode::OpenMemoryPoint`].
    pub fn encryption_key(&mut self, key: &'a CStr) -> &mut Self {
        self.info.encryptionkey = key.as_ptr();
        self
    }

    /// Maximum voice count for [`SoundType::Midi`] / [`SoundType::It`].
    pub fn max_polyphony(&mut self, polyphony: i32) -> &mut Self {
        self.info.maxpolyphony = polyphony;
        self
    }

    /// Attempt to load using the specified type first instead of loading in
    /// codec priority order.
    pub fn suggested_sound_type(&mut self, sound_type: SoundType) -> &mut Self {
        self.info.suggestedsoundtype = sound_type.into_raw();
        self
    }

    /// Callbacks for file operations.
    pub fn file_system<FS: AsyncListenFileSystem>(&mut self) -> &mut Self {
        self.info.fileuseropen = Some(useropen_listen::<FS>);
        self.info.fileuserclose = Some(userclose_listen::<FS>);
        self.info.fileuserread = Some(userread_listen::<FS>);
        self.info.fileuserseek = Some(userseek_listen::<FS>);
        self.info.fileuserasyncread = Some(userasyncread_listen::<FS>);
        self.info.fileuserasynccancel = Some(userasynccancel_listen::<FS>);
        self
    }

    /// Buffer size for reading the file, -1 to disable buffering.
    pub fn file_buffer_size(&mut self, size: i32) -> &mut Self {
        self.info.filebuffersize = size;
        self
    }

    /// Custom ordering of speakers for this sound data.
    pub fn channel_order(&mut self, order: ChannelOrder) -> &mut Self {
        self.info.channelorder = order.into_raw();
        self
    }

    /// SoundGroup to place this Sound in once created.
    pub fn initial_sound_group(&mut self, group: &'a SoundGroup) -> &mut Self {
        self.info.initialsoundgroup = group.as_raw();
        self
    }

    /// Initial position to seek to for [`Mode::CreateStream`].
    pub fn initial_seek_position(&mut self, position: u32, unit: TimeUnit) -> &mut Self {
        self.info.initialseekposition = position;
        self.info.initialseekpostype = unit.into_raw();
        self
    }

    /// Ignore file callbacks from [`System::set_file_system`] and this ex info.
    pub fn ignore_set_filesystem(&mut self) -> &mut Self {
        self.info.ignoresetfilesystem = 1;
        self
    }

    /// Hardware / software decoding policy for [`SoundType::AudioQueue`].
    pub fn audio_queue_policy(&mut self, policy: AudioQueueCodecPolicy) -> &mut Self {
        self.info.audioqueuepolicy = policy.into_raw() as u32;
        self
    }

    /// Mixer granularity for [SoundType::Midi] sounds, smaller numbers give a
    /// more accurate reproduction at the cost of higher CPU usage.
    pub fn min_midi_granularity(&mut self, granularity: u32) -> &mut Self {
        self.info.minmidigranularity = granularity;
        self
    }

    /// Thread index to execute [Mode::Nonblocking] loads on for parallel
    /// `Sound` loading.
    pub fn non_block_tread_id(&mut self, thread_id: i32) -> &mut Self {
        self.info.nonblockthreadid = thread_id;
        self
    }

    /// GUID of already loaded [`SoundType::Fsb`] file to reduce disk access.
    pub fn fsb_guid(&mut self, guid: &'a Guid) -> &mut Self {
        self.info.fsbguid = guid.as_raw() as *const FMOD_GUID as *mut FMOD_GUID;
        self
    }
}

macro_rules! debug_impl {
    ($($raw:ident, $rust:ident;)*) => {
        impl fmt::Debug for CreateSoundEx<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut d = f.debug_struct("CreateSoundEx");
                $(
                    if self.info.$raw != unsafe { std::mem::zeroed() } {
                        d.field(stringify!($rust), &self.info.$raw);
                    }
                )*
                d.finish_non_exhaustive()
            }
        }
    };
}

debug_impl! {
    length,                 length;
    fileoffset,             file_offset;
    numchannels,            num_channels;
    defaultfrequency,       default_frequency;
    format,                 format;
    decodebuffersize,       decode_buffer_size;
    initialsubsound,        initial_sub_sound;
    numsubsounds,           num_sub_sounds;
    maxpolyphony,           max_polyphony;
    suggestedsoundtype,     suggested_sound_type;
    filebuffersize,         file_buffer_size;
    channelorder,           channel_order;
    initialseekposition,    initial_seek_position;
    initialseekpostype,     initial_seek_position_type;
    ignoresetfilesystem,    ignore_set_filesystem;
    audioqueuepolicy,       audio_queue_policy;
    minmidigranularity,     min_midi_granularity;
    nonblockthreadid,       non_block_tread_id;
}
