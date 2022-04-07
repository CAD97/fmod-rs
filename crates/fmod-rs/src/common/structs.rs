// FMOD_ASYNCREADINFO

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Attributes3d {
    pub position: Vector,
    pub velocity: Vector,
    pub forward: Vector,
    pub up: Vector,
}

// FMOD_GUID
// FMOD_PLUGINLIST
// FMOD_ADVANCEDSETTINGS
// FMOD_TAG
// FMOD_CREATESOUNDEXINFO

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ReverbProperties {
    pub decay_time: f32,
    pub early_delay: f32,
    pub late_delay: f32,
    pub hf_reference: f32,
    pub hf_decay_ratio: f32,
    pub diffusion: f32,
    pub density: f32,
    pub low_shelf_frequency: f32,
    pub low_shelf_gain: f32,
    pub high_cut: f32,
    pub early_late_mix: f32,
    pub wet_level: f32,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuUsage {
    pub dsp: f32,
    pub stream: f32,
    pub geometry: f32,
    pub update: f32,
    pub convolution1: f32,
    pub convolution2: f32,
}
