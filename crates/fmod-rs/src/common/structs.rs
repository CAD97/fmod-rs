#[cfg(doc)]
use fmod::{InitFlags, Sound, System, ThreadType};
use {
    crate::utils::{decode_sbcd_u8, string_from_utf16be_lossy, string_from_utf16le_lossy},
    fmod::{raw::*, Error, Result, TagDataType, TagType},
    std::{borrow::Cow, ffi::CStr, slice},
};

macro_rules! fmod_struct {
    {$(
        $(#[$meta:meta])*
        $vis:vis struct $Name:ident = $Raw:ident {
            $($body:tt)*
        }
    )*} => {$(
        #[repr(C)]
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $Name {
            $($body)*
        }

        ::static_assertions::assert_eq_size!($Name, $Raw);
        ::static_assertions::assert_eq_align!($Name, $Raw);

        impl $Name {
            raw! {
                pub const fn from_raw(raw: $Raw) -> $Name {
                    unsafe { ::std::mem::transmute(raw) }
                }
            }
            raw! {
                pub const fn from_raw_ref(raw: &$Raw) -> &$Name {
                    unsafe { &*(raw as *const $Raw as *const $Name ) }
                }
            }
            raw! {
                pub fn from_raw_ref_mut(raw: &mut $Raw) -> &mut $Name {
                    unsafe { &mut *(raw as *mut $Raw as *mut $Name ) }
                }
            }
            raw! {
                pub const fn into_raw(self) -> $Raw {
                    unsafe { ::std::mem::transmute(self) }
                }
            }
            raw! {
                pub const fn as_raw(&self) -> &$Raw {
                    unsafe { &*(self as *const $Name as *const $Raw ) }
                }
            }
            raw! {
                pub fn as_raw_mut(&mut self) -> &mut $Raw {
                    unsafe { &mut *(self as *mut $Name as *mut $Raw ) }
                }
            }
        }
    )*};
}

/// FMOD version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Product version
    pub product: u8,
    /// Major version
    pub major: u8,
    /// Minor version
    pub minor: u8,
}

impl Version {
    raw! {
        #[allow(clippy::identity_op)]
        pub const fn from_raw(raw: u32) -> Version {
            Version {
                product: decode_sbcd_u8(((raw & 0x000000FF) >> 0) as u8),
                major: decode_sbcd_u8(((raw & 0x0000FF00) >> 4) as u8),
                minor: decode_sbcd_u8(((raw & 0x00FF0000) >> 8) as u8),
            }
        }
    }
}

// FMOD_ASYNCREADINFO

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
// FMOD_ADVANCEDSETTINGS

/// Tag data / metadata description.
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
                    if cfg!(debug_assertions) {
                        unreachable!("unknown {type:?} (len {}) encountered", tag.datalen)
                    }
                    #[cfg(feature = "tracing")]
                    tracing::error!(tag.datatype, tag.datalen, tag.data = ?data, "Unknown {type:?} encountered");
                    return Err(Error::InternalRs);
                },
            };
            Ok(Tag {
                kind: TagType::from_raw(tag.type_),
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
        /// **Units**: Milliseconds
        /// **Generic**: 1500
        /// **Range**: [0, 20000]
        pub decay_time: f32,
        /// Initial reflection delay time.
        ///
        /// **Units**: Milliseconds
        /// **Generic**: 7
        /// **Range**: [0, 300]
        pub early_delay: f32,
        /// Late reverberation delay time relative to initial reflection.
        ///
        /// ***Units**: Milliseconds
        /// **Generic**: 11
        /// **Range**: [0, 100]
        pub late_delay: f32,
        /// Reference high frequency.
        ///
        /// **Units**: Hertz
        /// **Generic**: 5000
        /// **Range**: [20, 20000]
        pub hf_reference: f32,
        /// High-frequency to mid-frequency decay time ratio.
        ///
        /// **Units**: Percent
        /// **Generic**: 50,
        /// **Range**: [10, 100]
        pub hf_decay_ratio: f32,
        /// Value that controls the echo density in the late reverberation decay.
        ///
        /// **Units**: Percent
        /// **Generic**: 50
        /// **Range**: [10, 100]
        pub diffusion: f32,
        /// Value that controls the modal density in the late reverberation decay.
        ///
        /// **Units**: Percent
        /// **Generic**: 100
        /// **Range**: [0, 100]
        pub density: f32,
        /// Reference low frequency
        ///
        /// **Units**: Hertz
        /// **Generic**: 250
        /// **Range**: [20, 1000]
        pub low_shelf_frequency: f32,
        /// Relative room effect level at low frequencies.
        ///
        /// **Units**: Decibels
        /// **Generic**: 0
        /// **Range**: [-36, 12]
        pub low_shelf_gain: f32,
        /// Relative room effect level at high frequencies.
        ///
        /// **Units**: Hertz
        /// **Generic**: 200000
        /// **Range**: [0, 20000]
        pub high_cut: f32,
        /// Early reflections level relative to room effect.
        ///
        /// **Units**: Percent
        /// **Generic**: 50
        /// **Range**: [0, 100]
        pub early_late_mix: f32,
        /// Room effect level at mid frequencies.
        ///
        /// **Units**: Decibels
        /// **Generic**: -6
        /// **Range**: [-80, 20]
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
