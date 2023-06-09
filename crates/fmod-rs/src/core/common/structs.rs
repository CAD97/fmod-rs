use {
    crate::utils::{decode_sbcd_u8, string_from_utf16be_lossy, string_from_utf16le_lossy},
    fmod::{raw::*, *},
    std::{
        borrow::Cow,
        ffi::c_char,
        ffi::CStr,
        marker::PhantomData,
        mem::{self, MaybeUninit},
        pin::Pin,
        ptr, slice,
    },
};

/// FMOD version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Product version.
    pub product: u8,
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
}

impl Version {
    raw! {
        #[allow(clippy::identity_op)]
        pub const fn from_raw(raw: u32) -> Version {
            Version {
                product: decode_sbcd_u8(((raw & 0x00FF0000) >> 16) as u8),
                major: decode_sbcd_u8(((raw & 0x0000FF00) >> 8) as u8),
                minor: decode_sbcd_u8(((raw & 0x000000FF) >> 0) as u8),
            }
        }
    }
}

/// A handle to an FMOD asynchronous file read request, received from
/// [`file::AsyncFileSystem`].
///
/// When servicing the async read operation, read from
/// [`handle`](Self::handle) at the given [`offset`](Self::offset) for
/// [`size`](Self::size) bytes into [`buffer`](Self::buffer). Then call
/// [`done`](Self::done) with the number of bytes read and the [`Result`]
/// that matches the success of the operation.
///
/// # Safety
///
/// This structure must not be used after calling [`done`](Self::done) or
/// the read operation has been [`cancel`led](file::AsyncFileSystem::cancel).
#[derive(Debug)]
pub struct AsyncReadInfo<File> {
    raw: *mut FMOD_ASYNCREADINFO,
    _phantom: PhantomData<*mut *mut File>,
}

unsafe impl<File> Send for AsyncReadInfo<File> where for<'a> &'a mut File: Send {}
unsafe impl<File> Sync for AsyncReadInfo<File> where for<'a> &'a mut File: Sync {}

#[allow(clippy::missing_safety_doc)]
impl<File> AsyncReadInfo<File> {
    raw! {
        pub const fn from_raw(raw: *mut FMOD_ASYNCREADINFO) -> Self {
            Self { raw, _phantom: PhantomData }
        }
    }

    raw! {
        pub const fn into_raw(self) -> *mut FMOD_ASYNCREADINFO {
            self.raw
        }
    }

    /// File handle that was provided by [`FileSystem::open`].
    pub unsafe fn handle<'a>(self) -> Pin<&'a File> {
        Pin::new_unchecked(&*(*self.raw).handle.cast())
    }

    /// File handle that was provided by [`FileSystem::open`].
    pub unsafe fn handle_mut<'a>(self) -> Pin<&'a mut File> {
        Pin::new_unchecked(&mut *(*self.raw).handle.cast())
    }

    /// Byte offset within the file where the read operation should occur.
    pub unsafe fn offset(self) -> u32 {
        (*self.raw).offset
    }

    /// Number of bytes to read.
    pub unsafe fn size(self) -> u32 {
        (*self.raw).sizebytes
    }

    /// Priority hint for how quickly this operation should be serviced
    /// where 0 represents low importance and 100 represents extreme
    /// importance. This could be used to prioritize the read order of a
    /// file job queue for example. FMOD decides the importance of the read
    /// based on if it could degrade audio or not.
    pub unsafe fn priority(self) -> i32 {
        (*self.raw).priority
    }

    /// Buffer to read data into.
    pub unsafe fn buffer(self) -> *mut [MaybeUninit<u8>] {
        let ptr = (*self.raw).buffer;
        let len = self.size();
        ptr::slice_from_raw_parts_mut(ptr.cast(), len as usize)
    }

    /// Completion function to signal the async read is done.
    ///
    /// Relevant result codes to use with this function include:
    ///
    /// - `Ok`: Read was successful.
    /// - [`Error::FileDiskEjected`]: Read was cancelled before being serviced.
    /// - [`Error::FileBad`]: Read operation failed for any other reason.
    pub unsafe fn done(self, result: Result<u32>) {
        let done = (*self.raw).done.unwrap_unchecked();
        match result {
            Ok(bytes_read) => {
                (*self.raw).bytesread = bytes_read;
                if bytes_read < self.size() {
                    done(self.raw, FMOD_ERR_FILE_EOF);
                } else {
                    done(self.raw, FMOD_OK);
                }
            },
            Err(err) => done(self.raw, err.into_raw()),
        }
    }
}

impl<File> Copy for AsyncReadInfo<File> {}
impl<File> Clone for AsyncReadInfo<File> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<File> Eq for AsyncReadInfo<File> {}
impl<File> PartialEq for AsyncReadInfo<File> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

fmod_struct! {
    /// Structure describing a point in 3D space.
    ///
    /// FMOD uses a left handed coordinate system by default.
    ///
    /// To use a right handed coordinate system specify [InitFlags::RightHanded3d] in [System::init].
    pub struct Vector = FMOD_VECTOR {
        /// X coordinate (right) in 3D space.
        pub x: f32,
        /// Y coordinate (up) in 3D space.
        pub y: f32,
        /// Z coordinate in 3D space.
        pub z: f32,
    }

    /// Structure describing a position, velocity and orientation.
    ///
    /// Vectors must be provided in the correct handedness.
    pub struct Attributes3d = FMOD_3D_ATTRIBUTES {
        /// Position in world space used for panning and attenuation.
        ///
        /// **Units**: Distance units
        pub position: Vector,
        /// Velocity in world space used for doppler.
        ///
        /// **Units**: Distance units per second
        pub velocity: Vector,
        /// Forwards orientation, must be of unit length (1.0) and perpendicular to `up`.
        pub forward: Vector,
        /// Upwards orientation, must be of unit length (1.0) and perpendicular to `forward`.
        pub up: Vector,
    }

    /// Structure describing a globally unique identifier.
    #[derive(Eq, Hash)]
    pub struct Guid = FMOD_GUID {
        /// Specifies the first 8 hexadecimal digits of the GUID.
        pub data1: u32,
        /// Specifies the first group of 4 hexadecimal digits.
        pub data2: u16,
        /// Specifies the second group of 4 hexadecimal digits.
        pub data3: u16,
        /// Array of 8 bytes. The first 2 bytes contain the third group of 4 hexadecimal digits. The remaining 6 bytes contain the final 12 hexadecimal digits.
        pub data4: [u8; 8],
    }
}

// FMOD_PLUGINLIST

fmod_struct! {
    /// Advanced configuration settings.
    ///
    /// Structure to allow configuration of lesser used system level settings.
    /// These tweaks generally allow the user to set resource limits and
    /// customize settings to better fit their application.
    ///
    /// 0 means to not change the setting (and this is provided by `default()`),
    /// so setting only a few members is a common use pattern.
    ///
    /// Specifying one of the codec maximums will help determine the maximum CPU
    /// usage of playing [Mode::CreateCompressedSample] Sounds of that type as well
    /// as the memory requirements. Memory will be allocated for 'up front' (during
    /// [System::init]) if these values are specified as non zero. If any are zero,
    /// it allocates memory for the codec whenever a file of the type in question is
    /// loaded. So if `max_mpeg_codecs` is 0 for example, it will allocate memory
    /// for the MPEG codecs the first time an MP3 is loaded or an MP3 based .FSB
    /// file is loaded.
    ///
    /// Setting `dsp_buffer_pool_size` will pre-allocate memory for the FMOD DSP
    /// network. See [DSP architecture guide]. By default 8 buffers are created up
    /// front. A large network might require more if the aim is to avoid real-time
    /// allocations from the FMOD mixer thread.
    ///
    /// [DSP architecture guide]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-dsp-architecture.html
    pub struct AdvancedSettings = FMOD_ADVANCEDSETTINGS {
        /// Size of this structure. Must be set to `size_of::<Self>()`.
        #[default(mem::size_of::<Self>() as i32)]
        size: i32,
        /// Maximum MPEG Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_mpeg_codecs: i32,
        /// Maximum IMA-ADPCM Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_adpcm_codecs: i32,
        /// Maximum XMA Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_xma_codecs: i32,
        /// Maximum Vorbis Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_vorbix_codecs: i32,
        /// Maximum AT9 Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_at9_codecs: i32,
        /// Maximum FADPCM Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_fadpcm_codecs: i32,
        /// Deprecated.
        max_pcm_codecs: i32,
        /// Number of elements in `asio_speaker_list` on input, number of elements
        /// in `asio_channel_list` on output.
        /// <dl>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        asio_num_channels: i32,
        /// Read only list of strings representing ASIO channel names, count is
        /// defined by `asio_num_channels`. Only valid after [System::init].
        #[default(ptr::null_mut())]
        asio_channel_list: *mut *mut c_char,
        /// List of speakers that represent each ASIO channel used for remapping,
        /// count is defined by `asio_num_channels`. Use [Speaker::None] to indicate
        /// no output for a given speaker.
        #[default(ptr::null_mut())]
        asio_speaker_list: *mut FMOD_SPEAKER,
        /// For use with [InitFlags::Vol0BecomesVirtual], [Channel]s with audibility
        /// below this will become virtual. See the [Virtual Voices] guide for more
        /// information.
        ///
        /// [Virtual Voices]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-virtual-voices.html
        /// <dl>
        /// <dt>Units</dt><dd>Linear</dd>
        /// <dt>Default</dt><dd>0</dd>
        /// </dl>
        pub vol_0_virtual_vol: f32,
        /// For use with Streams, the default size of the double buffer.
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>400</dd>
        /// <dt>Range</dt><dd>[0, 30000]</dd>
        /// </dl>
        pub default_decode_buffer_size: u32,
        /// For use with [InitFlags::ProfileEnable], specify the port to listen on
        /// for connections by FMOD Studio or FMOD Profiler.
        /// <dl>
        /// <dt>Default</dt><dd>9264</dd>
        /// </dl>
        pub profile_port: u16,
        /// For use with [Geometry], the maximum time it takes for a [Channel] to
        /// fade to the new volume level when its occlusion changes.
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>500</dd>
        /// </dl>
        pub geometry_max_fade_time: u32,
        /// For use with [InitFlags::ChannelDistanceFilter], the default center
        /// frequency for the distance filtering effect.
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>1500</dd>
        /// <dt>Range</dt><dd>[10, 22050]</dd>
        /// </dl>
        pub distance_filter_center_freq: f32,
        /// For use with [Reverb3D], selects which global reverb instance to use.
        /// <dl>
        /// <dt>Range</dt><dd>[0, MAX_INSTANCES]</dd>
        /// </dl>
        pub reverb_3d_instance: i32,
        /// Number of intermediate mixing buffers in the 'DSP buffer pool'. Each
        /// buffer in bytes will be `buffer_length` (See [System::get_dsp_buffer_size])
        /// × `size_of::<f32>()` × output mode speaker count (See [SpeakerMode]).
        /// ie 7.1 @ 1024 DSP block size = 1024 × 4 × 8 = 32KB.
        /// <dl>
        /// <dt>Default</dt><dd>8</dd>
        /// </dl>
        pub dsp_buffer_pool_size: i32,
        /// Resampling method used by [Channel]s.
        pub resampler_method: DspResampler,
        /// Seed value to initialize the internal random number generator.
        pub random_seed: u32,
        /// Maximum number of CPU threads to use for [DspType::Convolutionreverb]
        /// effect. 1 = effect is entirely processed inside the [ThreadType::Mixer]
        /// thread. 2 and 3 offloads different parts of the convolution processing
        /// into different threads ([ThreadType::Convolution1] and
        /// [ThreadType::Convolution2] to increase throughput.
        /// <dl>
        /// <dt>Default</dt><dd>3</dd>
        /// <dt>Range</dt><dd>[0, 3]</dd>
        /// </dl>
        pub max_convolution_threads: i32,
        /// Maximum Opus Sounds created as [Mode::CreateCompressedSample].
        /// <dl>
        /// <dt>Default</dt><dd>32</dd>
        /// <dt>Range</dt><dd>[0, 256]</dd>
        /// </dl>
        pub max_opus_codecs: i32,
    }
}

/// Tag data / metadata description.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag<'a> {
    /// Tag type.
    pub kind: TagType,
    /// Name.
    pub name: Cow<'a, str>,
    /// Tag data.
    pub data: TagData<'a>,
    /// True if this tag has been updated since last being accessed with [Sound::get_tag]
    pub updated: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagData<'a> {
    Binary(&'a [u8]),
    Int(i64),
    Float(f64),
    Str(Cow<'a, str>),
}

impl Tag<'_> {
    raw! {
        pub unsafe fn from_raw(tag: FMOD_TAG) -> Result<Self> {
            let name = CStr::from_ptr(tag.name);
            let name = name.to_string_lossy();
            let data = slice::from_raw_parts(tag.data as *const u8, tag.datalen as usize);
            let data = match TagDataType::from_raw(tag.datatype) {
                TagDataType::Binary => TagData::Binary(data),
                TagDataType::Int if data.len() == 1 => TagData::Int((tag.data as *const u8).read() as _),
                TagDataType::Int if data.len() == 2 => TagData::Int((tag.data as *const u16).read_unaligned() as _),
                TagDataType::Int if data.len() == 4 => TagData::Int((tag.data as *const u32).read_unaligned() as _),
                TagDataType::Int if data.len() == 8 => TagData::Int((tag.data as *const u64).read_unaligned() as _),
                TagDataType::Float if data.len() == 4 => TagData::Float((tag.data as *const f32).read_unaligned() as _),
                TagDataType::Float if data.len() == 8 => TagData::Float((tag.data as *const f64).read_unaligned() as _),
                TagDataType::String | TagDataType::StringUtf8 => TagData::Str(String::from_utf8_lossy(data)),
                TagDataType::StringUtf16 => TagData::Str(Cow::Owned(string_from_utf16le_lossy(data))),
                TagDataType::StringUtf16be => TagData::Str(Cow::Owned(string_from_utf16be_lossy(data))),
                r#type => {
                    whoops!(no_panic: "unknown {type:?} (len {}) encountered", tag.datalen);
                    return Err(Error::RustPanicked);
                },
            };
            Ok(Tag {
                kind: TagType::from_raw(tag.r#type),
                name,
                data,
                updated: tag.updated == 0,
            })
        }
    }
}

// FMOD_CREATESOUNDEXINFO

fmod_struct! {
    /// Structure defining a reverb environment.
    ///
    /// The generic reverb properties are those used by [ReverbProperties::GENERIC].
    pub struct ReverbProperties = FMOD_REVERB_PROPERTIES {
        /// Reverberation decay time.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>1500</dd>
        /// <dt>Range</dt><dd>[0, 20000]</dd>
        /// </dl>
        #[default(1500.0)]
        pub decay_time: f32,
        /// Initial reflection delay time.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>7</dd>
        /// <dt>Range</dt><dd>[0, 300]</dd>
        /// </dl>
        #[default(7.0)]
        pub early_delay: f32,
        /// Late reverberation delay time relative to initial reflection.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Milliseconds</dd>
        /// <dt>Default</dt><dd>11</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(11.0)]
        pub late_delay: f32,
        /// Reference high frequency.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>5000</dd>
        /// <dt>Range</dt><dd>[20, 20000]</dd>
        /// </dl>
        #[default(5000.0)]
        pub hf_reference: f32,
        /// High-frequency to mid-frequency decay time ratio.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[10, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub hf_decay_ratio: f32,
        /// Value that controls the echo density in the late reverberation decay.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[10, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub diffusion: f32,
        /// Value that controls the modal density in the late reverberation decay.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>100</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(100.0)]
        pub density: f32,
        /// Reference low frequency
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>250</dd>
        /// <dt>Range</dt><dd>[20, 1000]</dd>
        /// </dl>
        #[default(250.0)]
        pub low_shelf_frequency: f32,
        /// Relative room effect level at low frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Decibels</dd>
        /// <dt>Default</dt><dd>0</dd>
        /// <dt>Range</dt><dd>[-36, 12]</dd>
        /// </dl>
        #[default(0.0)]
        pub low_shelf_gain: f32,
        /// Relative room effect level at high frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Hertz</dd>
        /// <dt>Default</dt><dd>200000</dd>
        /// <dt>Range</dt><dd>[0, 20000]</dd>
        /// </dl>
        #[default(200000.0)]
        pub high_cut: f32,
        /// Early reflections level relative to room effect.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Percent</dd>
        /// <dt>Default</dt><dd>50</dd>
        /// <dt>Range</dt><dd>[0, 100]</dd>
        /// </dl>
        #[default(50.0)]
        pub early_late_mix: f32,
        /// Room effect level at mid frequencies.
        ///
        /// <dl>
        /// <dt>Units</dt><dd>Decibels</dd>
        /// <dt>Default</dt><dd>-6</dd>
        /// <dt>Range</dt><dd>[-80, 20]</dd>
        /// </dl>
        #[default(-6.0)]
        pub wet_level: f32,
    }
}

macro_rules! reverb {
    {
        $decay_time:expr,
        $early_delay:expr,
        $late_delay:expr,
        $hf_reference:expr,
        $hf_decay_ratio:expr,
        $diffusion:expr,
        $density:expr,
        $low_shelf_frequency:expr,
        $low_shelf_gain:expr,
        $high_cut:expr,
        $early_late_mix:expr,
        $wet_level:expr $(,)?
    } => {
        ReverbProperties {
            decay_time: $decay_time,
            early_delay: $early_delay,
            late_delay: $late_delay,
            hf_reference: $hf_reference,
            hf_decay_ratio: $hf_decay_ratio,
            diffusion: $diffusion,
            density: $density,
            low_shelf_frequency: $low_shelf_frequency,
            low_shelf_gain: $low_shelf_gain,
            high_cut: $high_cut,
            early_late_mix: $early_late_mix,
            wet_level: $wet_level,
        }
    };
}

#[rustfmt::skip]
impl ReverbProperties {
    pub const OFF: Self =               reverb! {  1000.0,    7.0,  11.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0,    20.0,  96.0, -80.0 };
    pub const GENERIC: Self =           reverb! {  1500.0,    7.0,  11.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0, 14500.0,  96.0,  -8.0 };
    pub const PADDEDCELL: Self =        reverb! {   170.0,    1.0,   2.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   160.0,  84.0,  -7.8 };
    pub const ROOM: Self =              reverb! {   400.0,    2.0,   3.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0,  6050.0,  88.0,  -9.4 };
    pub const BATHROOM: Self =          reverb! {  1500.0,    7.0,  11.0, 5000.0,  54.0, 100.0,  60.0, 250.0, 0.0,  2900.0,  83.0,   0.5 };
    pub const LIVINGROOM: Self =        reverb! {   500.0,    3.0,   4.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   160.0,  58.0, -19.0 };
    pub const STONEROOM: Self =         reverb! {  2300.0,   12.0,  17.0, 5000.0,  64.0, 100.0, 100.0, 250.0, 0.0,  7800.0,  71.0,  -8.5 };
    pub const AUDITORIUM: Self =        reverb! {  4300.0,   20.0,  30.0, 5000.0,  59.0, 100.0, 100.0, 250.0, 0.0,  5850.0,  64.0, -11.7 };
    pub const CONCERTHALL: Self =       reverb! {  3900.0,   20.0,  29.0, 5000.0,  70.0, 100.0, 100.0, 250.0, 0.0,  5650.0,  80.0,  -9.8 };
    pub const CAVE: Self =              reverb! {  2900.0,   15.0,  22.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0, 20000.0,  59.0, -11.3 };
    pub const ARENA: Self =             reverb! {  7200.0,   20.0,  30.0, 5000.0,  33.0, 100.0, 100.0, 250.0, 0.0,  4500.0,  80.0,  -9.6 };
    pub const HANGAR: Self =            reverb! { 10000.0,   20.0,  30.0, 5000.0,  23.0, 100.0, 100.0, 250.0, 0.0,  3400.0,  72.0,  -7.4 };
    pub const CARPETTEDHALLWAY: Self =  reverb! {   300.0,    2.0,  30.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   500.0,  56.0, -24.0 };
    pub const HALLWAY: Self =           reverb! {  1500.0,    7.0,  11.0, 5000.0,  59.0, 100.0, 100.0, 250.0, 0.0,  7800.0,  87.0,  -5.5 };
    pub const STONECORRIDOR: Self =     reverb! {   270.0,   13.0,  20.0, 5000.0,  79.0, 100.0, 100.0, 250.0, 0.0,  9000.0,  86.0,  -6.0 };
    pub const ALLEY: Self =             reverb! {  1500.0,    7.0,  11.0, 5000.0,  86.0, 100.0, 100.0, 250.0, 0.0,  8300.0,  80.0,  -9.8 };
    pub const FOREST: Self =            reverb! {  1500.0,  162.0,  88.0, 5000.0,  54.0,  79.0, 100.0, 250.0, 0.0,   760.0,  94.0, -12.3 };
    pub const CITY: Self =              reverb! {  1500.0,    7.0,  11.0, 5000.0,  67.0,  50.0, 100.0, 250.0, 0.0,  4050.0,  66.0, -26.0 };
    pub const MOUNTAINS: Self =         reverb! {  1500.0,  300.0, 100.0, 5000.0,  21.0,  27.0, 100.0, 250.0, 0.0,  1220.0,  82.0, -24.0 };
    pub const QUARRY: Self =            reverb! {  1500.0,   61.0,  25.0, 5000.0,  83.0, 100.0, 100.0, 250.0, 0.0,  3400.0, 100.0,  -5.0 };
    pub const PLAIN: Self =             reverb! {  1500.0,  179.0, 100.0, 5000.0,  50.0,  21.0, 100.0, 250.0, 0.0,  1670.0,  65.0, -28.0 };
    pub const PARKINGLOT: Self =        reverb! {  1700.0,    8.0,  12.0, 5000.0, 100.0, 100.0, 100.0, 250.0, 0.0, 20000.0,  56.0, -19.5 };
    pub const SEWERPIPE: Self =         reverb! {  2800.0,   14.0,  21.0, 5000.0,  14.0,  80.0,  60.0, 250.0, 0.0,  3400.0,  66.0,   1.2 };
    pub const UNDERWATER: Self =        reverb! {  1500.0,    7.0,  11.0, 5000.0,  10.0, 100.0, 100.0, 250.0, 0.0,   500.0,  92.0,   7.0 };
}

/// The maximum number of global/physical reverb instances.
///
/// Each instance of a physical reverb is an instance of a [DspSfxReverb] dsp in
/// the mix graph. This is unrelated to the number of possible Reverb3D objects,
/// which is unlimited.
pub const REVERB_MAX_INSTANCES: usize = FMOD_REVERB_MAXINSTANCES as usize;

// FMOD_ERRORCALLBACK_INFO

fmod_struct! {
    /// Performance information for Core API functionality.
    ///
    /// This structure is filled in with [System::get_cpu_usage].
    ///
    /// For readability, the percentage values are smoothed to provide a more stable output.
    ///
    /// 'Percentage of main thread' in the descriptions above refers to the thread that the function is called from by the user.
    ///
    /// The use of [ThreadType::Convolution1] or [ThreadType::Convolution2] can be controlled with [AdvancedSettings::max_convolution_threads].
    pub struct CpuUsage = FMOD_CPU_USAGE {
        /// DSP mixing engine CPU usage. Percentage of [ThreadType::Mixer], or main thread if [InitFlags::MixFromUpdate] flag is used with [System::init].
        pub dsp: f32,
        /// Streaming engine CPU usage. Percentage of [ThreadType::Stream], or main thread if [InitFlags::StreamFromUpdate] flag is used with [System::init].
        pub stream: f32,
        /// Geometry engine CPU usage. Percentage of [ThreadType::Geometry].
        pub geometry: f32,
        /// [System::update] CPU usage. Percentage of main thread.
        pub update: f32,
        /// Convolution reverb processing thread #1 CPU usage. Percentage of [ThreadType::Convolution1].
        pub convolution1: f32,
        /// Convolution reverb processing thread #2 CPU usage. Percentage of [ThreadType::Convolution2].
        pub convolution2: f32,
    }
}

// FMOD_DSP_DATA_PARAMETER_INFO

/// 3D attenuation factors for the direct and reverb paths.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Occlusion {
    /// Occlusion factor for the direct path where 0 represents no occlusion and 1 represents full occlusion.
    pub direct: f32,
    /// Occlusion factor for the reverb path where 0 represents no occlusion and 1 represents full occlusion.
    pub reverb: f32,
}
