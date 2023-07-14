use {
    crate::utils::{string_from_utf16be_lossy, string_from_utf16le_lossy},
    fmod::{raw::*, *},
    smart_default::SmartDefault,
    std::{borrow::Cow, ffi::c_char, ffi::CStr, mem, ptr, slice},
};

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

impl AdvancedSettings {
    /// ASIO channel names. Only valid after [System::init].
    pub fn asio_channel_list(&self) -> Option<impl Iterator<Item = Cow<'_, str>>> {
        if self.asio_channel_list.is_null() {
            None
        } else {
            Some(
                unsafe {
                    slice::from_raw_parts(self.asio_channel_list, ix!(self.asio_num_channels))
                }
                .iter()
                .copied()
                .map(|ptr| unsafe { CStr::from_ptr(ptr) })
                .map(CStr::to_bytes)
                .map(String::from_utf8_lossy),
            )
        }
    }

    /// List of speakers that represent each ASIO channel used for remapping.
    pub fn asio_speaker_list(&self) -> Option<&[Speaker]> {
        if self.asio_speaker_list.is_null() {
            None
        } else {
            Some(unsafe {
                slice::from_raw_parts(self.asio_speaker_list.cast(), ix!(self.asio_num_channels))
            })
        }
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
            let data = slice::from_raw_parts(tag.data as *const u8, ix!(tag.datalen));
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
                    yeet!(Error::RustPanicked);
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

impl<'a> TagData<'a> {
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            TagData::Binary(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            TagData::Int(data) => Some(*data),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            TagData::Float(data) => Some(*data),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            TagData::Str(data) => Some(data),
            _ => None,
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

// FMOD_DSP_DATA_PARAMETER_INFO

/// 3D attenuation factors for the direct and reverb paths.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Occlusion {
    /// Occlusion factor for the direct path where 0 represents no occlusion and 1 represents full occlusion.
    pub direct: f32,
    /// Occlusion factor for the reverb path where 0 represents no occlusion and 1 represents full occlusion.
    pub reverb: f32,
}

/// Angles and attenuation levels of a 3D cone shape,
/// for simulated occlusion which is based on direction.
#[derive(Debug, Clone, Copy, SmartDefault, PartialEq)]
pub struct Cone3dSettings {
    /// Inside cone angle. This is the angle spread within which the sound
    /// is unattenuated.
    /// <dl>
    /// <dt>Units</dt><dd>Degrees</dd>
    /// <dt>Range</dt><dd>[0, <code>outside_angle</code></dd>
    /// <dt>Default</dt><dd>360</dd>
    /// </dl>
    #[default(360.0)]
    pub inside_angle: f32,
    /// Outside cone angle. This is the angle spread outside of which the sound
    /// is attenuated to its `outside_volume`.
    /// <dl>
    /// <dt>Units</dt><dd>Degrees</dd>
    /// <dt>Range</dt><dd>[<code>inside_angle</code>, 360]</dd>
    /// <dt>Default</dt><dd>360</dd>
    /// </dl>
    #[default(360.0)]
    pub outside_angle: f32,
    /// Cone outside volume.
    /// <dl>
    /// <dt>Units</dt><dd>Linear</dd>
    /// <dt>Range</dt><dd>[0, 1]</dd>
    /// <dt>Default</dt><dd>1</dd>
    /// </dl>
    #[default(1.0)]
    pub outside_volume: f32,
}
