use {
    crate::utils::{decode_sbcd_u16, decode_sbcd_u8},
    fmod::{raw::*, *},
    smart_default::SmartDefault,
};

fmod_struct! {
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
        /// Orientation, must be orthonormal.
        pub orientation: Orientation3d,
    }
}

/// Orthonormal basis vectors that indicate a 3D orientation.
///
/// Defaults to a unit orientation for the default left-handed coordinate system.
/// The default is a reverse orientation if interpreted as right-handed.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, SmartDefault)]
pub struct Orientation3d {
    /// Forwards orientation, must be of unit length (1.0) and perpendicular to `up`.
    #[default(Vector::Z)]
    pub forward: Vector,
    /// Upwards orientation, must be of unit length (1.0) and perpendicular to `forward`.
    #[default(Vector::Y)]
    pub up: Vector,
}

fmod_struct! {
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

fmod_class! {
    /// Named marker for a given point in time.
    ///
    /// For for more information on sync points see [Sync Points].
    ///
    /// [Sync Points]: https://fmod.com/docs/2.02/api/glossary.html#sync-points
    weak class SyncPoint = FMOD_SYNCPOINT;
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
}

#[allow(nonstandard_style)]
/// Creates a new vector.
pub fn Vector(x: f32, y: f32, z: f32) -> Vector {
    Vector { x, y, z }
}

impl Vector {
    /// Creates a new vector.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// The unit X vector (right) in 3D space.
    pub const X: Vector = Vector::new(1.0, 0.0, 0.0);
    /// The unit Y vector (up) in 3D space.
    pub const Y: Vector = Vector::new(0.0, 1.0, 0.0);
    /// The unit Z vector in 3D space.
    ///
    /// FMOD uses a left handed coordinate system by default, meaning
    /// that the Z axis points forwards, away from the listener.
    pub const Z: Vector = Vector::new(0.0, 0.0, 1.0);
}

impl From<[f32; 3]> for Vector {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self { x, y, z }
    }
}

#[cfg(feature = "mint")]
impl mint::IntoMint for Vector {
    type MintType = mint::Vector3<f32>;
}

#[cfg(feature = "mint")]
impl From<mint::Vector3<f32>> for Vector {
    fn from(v: mint::Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

#[cfg(feature = "mint")]
impl From<Vector> for mint::Vector3<f32> {
    fn from(v: Vector) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

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

// -------------------------------------------------------------------------------------------------

fmod_flags! {
    /// Flags that describe the speakers present in a given signal.
    pub struct ChannelMask: FMOD_CHANNELMASK {
        /// Front left channel.
        FrontLeft       = FMOD_CHANNELMASK_FRONT_LEFT,
        /// Front right channel.
        FrontRight      = FMOD_CHANNELMASK_FRONT_RIGHT,
        /// Front center channel.
        FrontCenter     = FMOD_CHANNELMASK_FRONT_CENTER,
        /// Low frequency channel.
        LowFrequency    = FMOD_CHANNELMASK_LOW_FREQUENCY,
        /// Surround left channel.
        SurroundLeft    = FMOD_CHANNELMASK_SURROUND_LEFT,
        /// Surround right channel.
        SurroundRight   = FMOD_CHANNELMASK_SURROUND_RIGHT,
        /// Back left channel.
        BackLeft        = FMOD_CHANNELMASK_BACK_LEFT,
        /// Back right channel.
        BackRight       = FMOD_CHANNELMASK_BACK_RIGHT,
        /// Back center channel, not represented in any [SpeakerMode].
        BackCenter      = FMOD_CHANNELMASK_BACK_CENTER,
        /// Mono channel mask.
        Mono            = FMOD_CHANNELMASK_MONO,
        /// Stereo channel mask.
        Stereo          = FMOD_CHANNELMASK_STEREO,
        /// Left / right / center channel mask.
        Lrc             = FMOD_CHANNELMASK_LRC,
        /// Quadphonic channel mask.
        Quad            = FMOD_CHANNELMASK_QUAD,
        /// 5.0 surround channel mask.
        Surround        = FMOD_CHANNELMASK_SURROUND,
        /// 5.1 surround channel mask.
        Surround51      = FMOD_CHANNELMASK_5POINT1,
        /// 5.1 surround channel mask, using rears instead of surrounds.
        Surround51Rears = FMOD_CHANNELMASK_5POINT1_REARS,
        /// 7.0 surround channel mask.
        Surround70      = FMOD_CHANNELMASK_7POINT0,
        /// 7.1 surround channel mask.
        Surround71      = FMOD_CHANNELMASK_7POINT1,
    }
}

fmod_enum! {
    /// Speaker ordering for multichannel signals.
    #[derive(Default)]
    pub enum ChannelOrder: FMOD_CHANNELORDER {
        #[default]
        /// Left, Right, Center, LFE, Surround Left, Surround Right, Back Left, Back Right (see [Speaker] enumeration)
        Default    = FMOD_CHANNELORDER_DEFAULT,
        /// Left, Right, Center, LFE, Back Left, Back Right, Surround Left, Surround Right (as per Microsoft .wav WAVEFORMAT structure master order)
        WaveFormat = FMOD_CHANNELORDER_WAVEFORMAT,
        /// Left, Center, Right, Surround Left, Surround Right, LFE
        ProTools   = FMOD_CHANNELORDER_PROTOOLS,
        /// Mono, Mono, Mono, Mono, Mono, Mono, ... (each channel up to [MAX_CHANNEL_WIDTH] treated as mono)
        AllMono    = FMOD_CHANNELORDER_ALLMONO,
        /// Left, Right, Left, Right, Left, Right, ... (each pair of channels up to [MAX_CHANNEL_WIDTH] treated as stereo)
        AllStereo  = FMOD_CHANNELORDER_ALLSTEREO,
        /// Left, Right, Surround Left, Surround Right, Center, LFE (as per Linux ALSA channel order)
        Alsa       = FMOD_CHANNELORDER_ALSA,
    }
}

/// Maximum number of channels per frame of audio supported by audio files,
/// buffers, connections and DSPs.
pub const MAX_CHANNEL_WIDTH: usize = FMOD_MAX_CHANNEL_WIDTH as usize;

/// Maximum number of listeners supported.
pub const MAX_LISTENERS: usize = FMOD_MAX_LISTENERS as usize;

/// Maximum number of System objects allowed.
pub const MAX_SYSTEMS: usize = FMOD_MAX_SYSTEMS as usize;

/// Current FMOD version number.
#[doc = concat!("(", env!("FMOD_VERSION"), ")")]
pub const VERSION: Version = Version::from_raw(raw::FMOD_VERSION);

/// FMOD version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Product version.
    pub product: u16,
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
}

impl Version {
    /// Creates a new version tuple.
    pub const fn new(product: u16, major: u8, minor: u8) -> Version {
        Version {
            product,
            major,
            minor,
        }
    }

    raw! {
        #[allow(clippy::identity_op)]
        pub const fn from_raw(raw: u32) -> Version {
            Version {
                product: decode_sbcd_u16(((raw & 0xFFFF0000) >> 16) as u16),
                major: decode_sbcd_u8(((raw & 0x0000FF00) >> 8) as u8),
                minor: decode_sbcd_u8(((raw & 0x000000FF) >> 0) as u8),
            }
        }
    }
}

fmod_flags! {
    /// Sound description bitfields, bitwise OR them together for loading and describing sounds.
    ///
    /// By default a sound will open as a static sound that is decompressed fully into memory to PCM. (ie equivalent of [Mode::CreateSample]) To have a sound stream instead, use [Mode::CreateStream], or use the wrapper function [System::create_stream].
    ///
    /// Some opening modes (ie [Mode::OpenUser], [Mode::OpenMemory], [Mode::OpenMemoryPoint], [Mode::OpenRaw]) will need extra information. This can be provided using the [CreateSoundExInfo] structure.
    ///
    /// Specifying [Mode::OpenMemoryPoint] will POINT to your memory rather allocating its own sound buffers and duplicating it internally. This means you cannot free the memory while FMOD is using it, until after the sound is released.
    ///
    /// With [Mode::OpenMemoryPoint], for PCM formats, only WAV, FSB, and RAW are supported. For compressed formats, only those formats supported by [Mode::CreateCompressedSample] are supported.
    ///
    /// With [Mode::OpenMemoryPoint] and [Mode::OpenRaw] or PCM, if using them together, note that you must pad the data on each side by 16 bytes. This is so fmod can modify the ends of the data for looping / interpolation / mixing purposes. If a wav file, you will need to insert silence, and then reset loop points to stop the playback from playing that silence.
    pub struct Mode: FMOD_MODE {
        #[default]
        /// Default for all modes listed below. [Mode::LoopOff], [Mode::D2], [Mode::WorldRelative3d], [Mode::InverseRolloff3d]
        Default                 = FMOD_DEFAULT,
        /// For non looping sounds. (DEFAULT). Overrides [Mode::LoopNormal] / [Mode::LoopBidi].
        LoopOff                 = FMOD_LOOP_OFF,
        /// For forward looping sounds.
        LoopNormal              = FMOD_LOOP_NORMAL,
        /// For bidirectional looping sounds. (only works on software mixed static sounds).
        LoopBidi                = FMOD_LOOP_BIDI,
        /// Ignores any 3d processing. (DEFAULT).
        D2                      = FMOD_2D,
        /// Makes the sound positionable in 3D. Overrides [Mode::D3].
        D3                      = FMOD_3D,
        /// Decompress at runtime, streaming from the source provided (ie from disk). Overrides [Mode::CreateSample] and [Mode::CreateCompressedSample]. Note a stream can only be played once at a time due to a stream only having 1 stream buffer and file handle. Open multiple streams to have them play concurrently.
        CreateStream            = FMOD_CREATESTREAM,
        /// Decompress at loadtime, decompressing or decoding whole file into memory as the target sample format (ie PCM). Fastest for playback and most flexible.
        CreateSample            = FMOD_CREATESAMPLE,
        /// Load MP2/MP3/FADPCM/IMAADPCM/Vorbis/AT9 or XMA into memory and leave it compressed. Vorbis/AT9/FADPCM encoding only supported in the .FSB container format. During playback the FMOD software mixer will decode it in realtime as a 'compressed sample'. Overrides [Mode::CreateSample]. If the sound data is not one of the supported formats, it will behave as if it was created with [Mode::CreateSample] and decode the sound into PCM.
        CreateCompressedSample  = FMOD_CREATECOMPRESSEDSAMPLE,
        /// Opens a user created static sample or stream. Use [CreateSoundExInfo] to specify format, defaultfrequency, numchannels, and optionally a read callback. If a user created 'sample' is created with no read callback, the sample will be empty. Use [Sound::lock] and [Sound::unlock] to place sound data into the sound if this is the case.
        OpenUser                = FMOD_OPENUSER,
        /// "name_or_data" will be interpreted as a pointer to memory instead of filename for creating sounds. Use [CreateSoundExInfo] to specify length. If used with [Mode::CreateSample] or [Mode::CreateCompressedSample], FMOD duplicates the memory into its own buffers. Your own buffer can be freed after open, unless you are using [Mode::NonBlocking] then wait until the Sound is in the [OpenState::Ready] state. If used with [Mode::CreateStream], FMOD will stream out of the buffer whose pointer you passed in. In this case, your own buffer should not be freed until you have finished with and released the stream.
        OpenMemory              = FMOD_OPENMEMORY,
        /// "name_or_data" will be interpreted as a pointer to memory instead of filename for creating sounds. Use [CreateSoundExInfo] to specify length. This differs to [Mode::OpenMemory] in that it uses the memory as is, without duplicating the memory into its own buffers. Cannot be freed after open, only after the sound is released. Will not work if the data is compressed and [Mode::CreateCompressedSample] is not used. Cannot be used in conjunction with [CreateSoundExInfo::encryption_key].
        OpenMemoryPoint         = FMOD_OPENMEMORY_POINT,
        /// Will ignore file format and treat as raw pcm. Use [CreateSoundExInfo] to specify format. Requires at least default_frequency, num_channels and format to be specified before it will open. Must be little endian data.
        OpenRaw                 = FMOD_OPENRAW,
        /// Just open the file, dont prebuffer or read. Good for fast opens for info, or when [Sound::read_data] is to be used.
        OpenOnly                = FMOD_OPENONLY,
        /// For [System::create_sound] - for accurate [Sound::get_length] / [Channel::set_position] on VBR MP3, and MOD/S3M/XM/IT/MIDI files. Scans file first, so takes longer to open. [Mode::OpenOnly] does not affect this.
        AccurateTime            = FMOD_ACCURATETIME,
        /// For corrupted / bad MP3 files. This will search all the way through the file until it hits a valid MPEG header. Normally only searches for 4k.
        MpegSearch              = FMOD_MPEGSEARCH,
        /// For opening sounds and getting streamed subsounds (seeking) asynchronously. Use [Sound::get_open_state] to poll the state of the sound as it opens or retrieves the subsound in the background.
        NonBlocking             = FMOD_NONBLOCKING,
        /// Unique sound, can only be played one at a time
        Unique                  = FMOD_UNIQUE,
        /// Make the sound's position, velocity and orientation relative to the listener.
        HeadRelative3d          = FMOD_3D_HEADRELATIVE,
        /// Make the sound's position, velocity and orientation absolute (relative to the world). (DEFAULT)
        WorldRelative3d         = FMOD_3D_WORLDRELATIVE,
        /// This sound will follow the inverse rolloff model where mindistance = full volume, maxdistance = where sound stops attenuating, and rolloff is fixed according to the global rolloff factor. (DEFAULT)
        InverseRolloff3d        = FMOD_3D_INVERSEROLLOFF,
        /// This sound will follow a linear rolloff model where mindistance = full volume, maxdistance = silence.
        LinearRolloff3d         = FMOD_3D_LINEARROLLOFF,
        /// This sound will follow a linear-square rolloff model where mindistance = full volume, maxdistance = silence.
        LinearSquareRolloff3d   = FMOD_3D_LINEARSQUAREROLLOFF,
        /// This sound will follow the inverse rolloff model at distances close to mindistance and a linear-square rolloff close to maxdistance.
        InverseTaperedRolloff3d = FMOD_3D_INVERSETAPEREDROLLOFF,
        /// This sound will follow a rolloff model defined by [Sound::set_3d_custom_rolloff] / [Channel::set_3d_custom_rolloff].
        CustomRolloff3d         = FMOD_3D_CUSTOMROLLOFF,
        /// Is not affect by geometry occlusion. If not specified in [Sound::set_mode], or [Channel::set_mode], the flag is cleared and it is affected by geometry again.
        IgnoreGeometry3d        = FMOD_3D_IGNOREGEOMETRY,
        /// Skips id3v2/asf/etc tag checks when opening a sound, to reduce seek/read overhead when opening files.
        IgnoreTags              = FMOD_IGNORETAGS,
        /// Removes some features from samples to give a lower memory overhead, like [Sound::get_name].
        LowMem                  = FMOD_LOWMEM,
        /// For sounds that start virtual (due to being quiet or low importance), instead of swapping back to audible, and playing at the correct offset according to time, this flag makes the sound play from the start.
        VirtualPlayFromStart    = FMOD_VIRTUAL_PLAYFROMSTART,
    }
}

fmod_enum! {
    /// Assigns an enumeration for a speaker index.
    pub enum Speaker: FMOD_SPEAKER
    where
        const { self >= FMOD_SPEAKER_NONE },
    {
        /// No speaker
        None          = FMOD_SPEAKER_NONE,
        /// The front left speaker
        FrontLeft     = FMOD_SPEAKER_FRONT_LEFT,
        /// The front right speaker
        FrontRight    = FMOD_SPEAKER_FRONT_RIGHT,
        /// The front center speaker
        FrontCenter   = FMOD_SPEAKER_FRONT_CENTER,
        /// The LFE or 'subwoofer' speaker
        LowFrequency  = FMOD_SPEAKER_LOW_FREQUENCY,
        /// The surround left (usually to the side) speaker
        SurroundLeft  = FMOD_SPEAKER_SURROUND_LEFT,
        /// The surround right (usually to the side) speaker
        SurroundRight = FMOD_SPEAKER_SURROUND_RIGHT,
        /// The back left speaker
        BackLeft      = FMOD_SPEAKER_BACK_LEFT,
        /// The back right speaker
        BackRight     = FMOD_SPEAKER_BACK_RIGHT,
        /// The top front left speaker
        TopFrontLeft  = FMOD_SPEAKER_TOP_FRONT_LEFT,
        /// The top front right speaker
        TopFrontRight = FMOD_SPEAKER_TOP_FRONT_RIGHT,
        /// The top back left speaker
        TopBackLeft   = FMOD_SPEAKER_TOP_BACK_LEFT,
        /// The top back right speaker
        TopBackRight  = FMOD_SPEAKER_TOP_BACK_RIGHT,
    }
}

fmod_enum! {
    /// Speaker mode types.
    ///
    /// Note below the phrase 'sound channels' is used. These are the subchannels inside a sound, they are not related and have nothing to do with the FMOD class "Channel".
    ///
    /// For example a mono sound has 1 sound channel, a stereo sound has 2 sound channels, and an AC3 or 6 channel wav file have 6 "sound channels".
    ///
    /// See the FMOD Studio Mixing Guide for graphical depictions of each speaker mode.
    #[derive(Default)]
    pub enum SpeakerMode: FMOD_SPEAKERMODE {
        #[default]
        /// Default speaker mode for the chosen output mode which will resolve after [System::init].
        Default     = FMOD_SPEAKERMODE_DEFAULT,
        /// Assume there is no special mapping from a given channel to a speaker, channels map 1:1 in order. Use [System::set_software_format] to specify the speaker count.
        ///
        /// This mode is for output devices that are not specifically mono/stereo/quad/surround/5.1 or 7.1, but are multichannel.
        ///
        /// - Use [System::set_software_format] to specify the number of speakers you want to address, otherwise it will default to 2 (stereo).
        /// - Sound channels map to speakers sequentially, so a mono sound maps to output speaker 0, stereo sound maps to output speaker 0 & 1.
        /// - The user assumes knowledge of the speaker order. [Speaker] enumerations may not apply, so raw channel indices should be used.
        /// - Multichannel sounds map input channels to output channels 1:1.
        /// - Speaker levels must be manually set with [Channel::set_mix_matrix].
        /// - [Channel::set_pan] and [Channel::set_mix_levels_output] do not work.
        Raw         = FMOD_SPEAKERMODE_RAW,
        /// 1 speaker setup (monaural).
        ///
        /// This mode is for a 1 speaker arrangement.
        ///
        /// - Panning does not work in this speaker mode.
        /// - Mono, stereo and multichannel sounds have each sound channel played on the one speaker at unity.
        /// - Mix behavior for multichannel sounds can be set with [Channel::set_mix_matrix].
        Mono        = FMOD_SPEAKERMODE_MONO,
        /// 2 speaker setup (stereo) front left, front right.
        Stereo      = FMOD_SPEAKERMODE_STEREO,
        /// 4 speaker setup (4.0) front left, front right, surround left, surround right.
        ///
        /// This mode is for 2 speaker arrangements that have a left and right speaker.
        ///
        /// - Mono sounds default to an even distribution between left and right. They can be panned with Channel::set_pan.
        /// - Stereo sounds default to the left sound channel played on the front left, and the right sound channel played on the front right. They can be cross faded with Channel::set_pan.
        /// - Multichannel sounds default to all of their sound channels being played on each speaker in order of input.
        /// - Mix behavior for multichannel sounds can be set with [Channel::set_mix_matrix].
        Quad        = FMOD_SPEAKERMODE_QUAD,
        /// 5 speaker setup (5.0) front left, front right, center, surround left, surround right.
        ///
        /// This mode is for 5 speaker arrangements that have a left/right/center/surround left/surround right.
        ///
        /// - Mono sounds default to the center speaker. They can be panned with [Channel::set_pan].
        /// - Stereo sounds default to the left sound channel played on the front left, and the right sound channel played on the front right. They can be cross faded with [Channel::set_pan].
        /// - Multichannel sounds default to all of their sound channels being played on each speaker in order of input.
        /// - Mix behavior for multichannel sounds can be set with [Channel::set_mix_matrix].
        Surround    = FMOD_SPEAKERMODE_SURROUND,
        /// 6 speaker setup (5.1) front left, front right, center, low frequency, surround left, surround right.
        ///
        /// This mode is for 5.1 speaker arrangements that have a left/right/center/surround left/surround right and a subwoofer speaker.
        ///
        /// - Mono sounds default to the center speaker. They can be panned with Channel::set_pan.
        /// - Stereo sounds default to the left sound channel played on the front left, and the right sound channel played on the front right. They can be cross faded with Channel::set_pan.
        /// - Multichannel sounds default to all of their sound channels being played on each speaker in order of input.
        /// - Mix behavior for multichannel sounds can be set with Channel::set_mix_matrix.
        Surround51  = FMOD_SPEAKERMODE_5POINT1,
        /// 8 speaker setup (7.1) front left, front right, center, low frequency, surround left, surround right, back left, back right.
        ///
        /// This mode is for 7.1 speaker arrangements that have a left/right/center/surround left/surround right/rear left/rear right and a subwoofer speaker.
        ///
        /// - Mono sounds default to the center speaker. They can be panned with Channel::set_pan.
        /// - Stereo sounds default to the left sound channel played on the front left, and the right sound channel played on the front right. They can be cross faded with Channel::set_pan.
        /// - Multichannel sounds default to all of their sound channels being played on each speaker in order of input.
        /// - Mix behavior for multichannel sounds can be set with Channel::set_mix_matrix.
        Surround71  = FMOD_SPEAKERMODE_7POINT1,
        /// 12 speaker setup (7.1.4) front left, front right, center, low frequency, surround left, surround right, back left, back right, top front left, top front right, top back left, top back right.
        Surround714 = FMOD_SPEAKERMODE_7POINT1POINT4,
    }
}

fmod_typedef! {
    /// Time types used for position or length.
    pub enum TimeUnit: FMOD_TIMEUNIT {
        /// Milliseconds.
        Ms              = FMOD_TIMEUNIT_MS,
        /// PCM samples, related to milliseconds * samplerate / 1000.
        Pcm             = FMOD_TIMEUNIT_PCM,
        /// Bytes, related to PCM samples * channels * datawidth (ie 16bit = 2 bytes).
        PcmBytes        = FMOD_TIMEUNIT_PCMBYTES,
        /// Raw file bytes of (compressed) sound data (does not include headers). Only used by [Sound::get_length] and [Channel::get_position].
        RawBytes        = FMOD_TIMEUNIT_RAWBYTES,
        /// Fractions of 1 PCM sample. Unsigned int range 0 to 0xFFFFFFFF. Used for sub-sample granularity for [Dsp] purposes.
        PcmFraction     = FMOD_TIMEUNIT_PCMFRACTION,
        /// MOD/S3M/XM/IT. Order in a sequenced module format. Use [Sound::get_format] to determine the PCM format being decoded to.
        ModOrder        = FMOD_TIMEUNIT_MODORDER,
        /// MOD/S3M/XM/IT. Current row in a sequenced module format. Cannot use with [Channel::set_position]. [Sound::get_length] will return the number of rows in the currently playing or seeked to pattern.
        ModRow          = FMOD_TIMEUNIT_MODROW,
        /// MOD/S3M/XM/IT. Current pattern in a sequenced module format. Cannot use with [Channel::set_position]. [Sound::get_length] will return the number of patterns in the song and {Channel::get_position} will return the currently playing pattern.
        ModPattern      = FMOD_TIMEUNIT_MODPATTERN,
    }
}

/// Time used for position or length.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, SmartDefault)]
pub struct Time {
    pub value: u32,
    #[default(TimeUnit::Pcm)]
    pub unit: TimeUnit,
}

impl Time {
    /// Create a new time measure.
    pub fn new(value: u32, unit: TimeUnit) -> Self {
        Self { value, unit }
    }

    /// Create a new time measure in [milliseconds](TimeUnit::Ms).
    pub fn ms(value: u32) -> Self {
        Self::new(value, TimeUnit::Ms)
    }

    /// Create a new time measure in [PCM samples](TimeUnit::Pcm).
    pub fn pcm(value: u32) -> Self {
        Self::new(value, TimeUnit::Pcm)
    }

    /// Create a new time measure in [PCM bytes](TimeUnit::PcmBytes).
    pub fn pcm_bytes(value: u32) -> Self {
        Self::new(value, TimeUnit::PcmBytes)
    }

    /// Create a new time measure in [raw bytes](TimeUnit::RawBytes).
    pub fn raw_bytes(value: u32) -> Self {
        Self::new(value, TimeUnit::RawBytes)
    }

    /// Create a new time measure in [PCM fractions](TimeUnit::PcmFraction).
    pub fn pcm_fraction(value: u32) -> Self {
        Self::new(value, TimeUnit::PcmFraction)
    }

    /// Create a new time measure in [MOD/S3M/XM/IT order](TimeUnit::ModOrder).
    pub fn mod_order(value: u32) -> Self {
        Self::new(value, TimeUnit::ModOrder)
    }

    /// Create a new time measure in [MOD/S3M/XM/IT row](TimeUnit::ModRow).
    pub fn mod_row(value: u32) -> Self {
        Self::new(value, TimeUnit::ModRow)
    }

    /// Create a new time measure in [MOD/S3M/XM/IT pattern](TimeUnit::ModPattern).
    pub fn mod_pattern(value: u32) -> Self {
        Self::new(value, TimeUnit::ModPattern)
    }
}

/// 3D attenuation factors for the direct and reverb paths.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Occlusion {
    /// Occlusion factor for the direct path where 0 represents no occlusion and 1 represents full occlusion.
    pub direct: f32,
    /// Occlusion factor for the reverb path where 0 represents no occlusion and 1 represents full occlusion.
    pub reverb: f32,
}
