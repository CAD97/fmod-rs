use {
    crate::utils::{string_from_utf16be_lossy, string_from_utf16le_lossy},
    fmod::{raw::*, *},
    smart_default::SmartDefault,
    std::{borrow::Cow, ffi::CStr, slice},
};

// FMOD_PLUGINLIST

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

// FMOD_DSP_DATA_PARAMETER_INFO

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
