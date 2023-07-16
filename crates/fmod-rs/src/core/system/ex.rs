use {fmod::*, smart_default::SmartDefault, std::mem::ManuallyDrop};

/// Identification information about a sound device.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DriverInfo {
    /// GUID that uniquely identifies the device.
    pub guid: Guid,
    /// Sample rate this device operates at.
    pub system_rate: i32,
    /// Speaker setup this device is currently using.
    pub speaker_mode: SpeakerMode,
    /// Number of channels in the current speaker setup.
    pub speaker_mode_channels: i32,
    /// Flags that provide additional information about the driver.
    /// Only meaningful for record drivers.
    pub state: DriverState,
}

/// Output format for the software mixer.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SoftwareFormat {
    /// Sample rate of the mixer.
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[8000, 192000]</dd>
    /// <dt>Units</dt><dd>Hertz</dd>
    /// <dt>Default</dt><dd>48000</dd>
    /// </dl>
    pub sample_rate: i32,
    /// Speaker setup of the mixer.
    pub speaker_mode: SpeakerMode,
    /// Number of speakers for [SpeakerMode::Raw].
    ///
    /// <dl>
    /// <dt>Range</dt><dd>[0, MAX_CHANNEL_WIDTH]</dd>
    /// </dl>
    pub num_raw_speakers: i32,
}

/// The buffer size for the FMOD software mixing engine.
#[derive(Debug, SmartDefault, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DspBufferSize {
    /// The mixer engine block size. Use this to adjust mixer update
    /// granularity. See below for more information on buffer length vs latency.
    ///
    /// <dl>
    /// <dt>Units</dt><dd>Samples</dd>
    /// <dt>Default</dt><dd>1024</dd>
    /// </dl>
    #[default(1024)]
    pub buffer_length: u32,
    /// The mixer engine number of buffers used. Use this to adjust mixer
    /// latency. See [System::set_dsp_buffer_size] for more information on
    /// number of buffers vs latency.
    pub num_buffers: i32,
}

/// The position of a speaker for the current speaker mode.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct SpeakerPosition {
    /// 2D X position relative to the listener. -1 = left, 0 = middle,
    /// +1 = right.
    /// <dl>
    /// <dt>Range</dt><dd>[-1, 1]</dd>
    /// </dl>
    pub x: f32,
    /// 2D Y position relative to the listener. -1 = back, 0 = middle,
    /// +1 = front.
    /// <dl>
    /// <dt>Range</dt><dd>[-1, 1]</dd>
    /// </dl>
    pub y: f32,
    /// Active state of a speaker. true = included in 3D calculations,
    /// false = ignored.
    pub active: bool,
}

/// The global doppler scale, distance factor and log rolloff scale for all 3D
/// sound in FMOD.
#[derive(Debug, SmartDefault, Copy, Clone, PartialEq)]
pub struct Settings3d {
    /// A general scaling factor for how much the pitch varies due to doppler
    /// shifting in 3D sound. Doppler is the pitch bending effect when a sound
    /// comes towards the listener or moves away from it, much like the effect
    /// you hear when a train goes past you with its horn sounding. With
    /// `doppler_scale` you can exaggerate or diminish the effect. FMOD's
    /// effective speed of sound at a doppler factor of 1.0 is 340 m/s.
    #[default(1.0)]
    pub doppler_scale: f32,
    /// The FMOD 3D engine relative distance factor, compared to 1.0 meters.
    /// Another way to put it is that it equates to "how many units per meter
    /// does your engine have". For example, if you are using feet then "scale"
    /// would equal 3.28.  
    /// This only affects doppler. If you keep your min/max distance, custom
    /// rolloff curves and positions in scale relative to each other the volume
    /// rolloff will not change. If you set this, the min_distance of a sound
    /// will automatically set itself to this value when it is created in case
    /// the user forgets to set the min_distance to match the new
    /// distance_factor.
    #[default(1.0)]
    pub distance_factor: f32,
    /// The global attenuation rolloff factor. Volume for a sound will scale at
    /// min_distance / distance. Setting this value makes the sound drop off
    /// faster or slower. The higher the value, the faster volume will
    /// attenuate, and conversely the lower the value, the slower it will
    /// attenuate. For example, a rolloff factor of 1 will simulate the real
    /// world, where as a value of 2 will make sounds attenuate 2 times quicker.
    #[default(1.0)]
    pub rolloff_scale: f32,
}

/// Information about a selected plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInfo {
    /// Plugin type.
    pub kind: PluginType,
    /// Version number of the plugin.
    pub version: u32,
}

/// A number of playing channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelUsage {
    /// Number of playing [Channel]s (both real and virtual).
    pub all: i32,
    /// Number of playing real (non-virtual) [Channel]s.
    pub real: i32,
}

/// Running total information about file reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileUsage {
    /// Total bytes read from file for loading sample data.
    pub sample_bytes_read: i64,
    /// Total bytes read from file for streaming sounds.
    pub stream_bytes_read: i64,
    /// Total bytes read for non-audio data such as FMOD Studio banks.
    pub other_bytes_read: i64,
}

/// Position, velocity, and orientation of a 3D sound listener.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ListenerAttributes3d {
    /// Position in 3D space used for panning and attenuation.
    pub pos: Vector,
    /// Velocity in 3D space used for doppler.
    pub vel: Vector,
    /// Forwards orientation.
    pub forward: Vector,
    /// Upwards orientation.
    pub up: Vector,
}

/// Number of recording devices available.
#[derive(Debug)]
pub struct NumDrivers {
    /// Number of recording drivers available for this output mode.
    pub available: i32,
    /// Number of recording driver currently plugged in.
    pub connected: i32,
}

/// Mutual exclusion lock guard for the FMOD DSP engine.
///
/// The lock is released when this guard is dropped.
pub struct DspLock<'a> {
    system: &'a System,
}

impl DspLock<'_> {
    /// Mutual exclusion function to lock the FMOD DSP engine (which runs
    /// asynchronously in another thread), so that it will not execute.
    ///
    /// See [`System::lock_dsp`] for more information.
    ///
    /// # Safety
    ///
    /// The DSP engine must not already be locked when this function is called.
    pub unsafe fn new(system: &System) -> Result<DspLock<'_>> {
        system.lock_dsp()?;
        Ok(DspLock { system })
    }

    /// Mutual exclusion function to unlock the FMOD DSP engine (which runs
    /// asynchronously in another thread) and let it continue executing.
    pub fn unlock(self) -> Result {
        let this = ManuallyDrop::new(self);
        unsafe { this.system.unlock_dsp() }
    }
}

impl Drop for DspLock<'_> {
    fn drop(&mut self) {
        match unsafe { self.system.unlock_dsp() } {
            Ok(()) => (),
            Err(e) => {
                whoops!("error unlocking DSP engine: {e}");
            },
        }
    }
}

#[cfg(windows)]
pub type SystemThreadHandle = std::os::windows::io::RawHandle;
#[cfg(unix)]
pub type SystemThreadHandle = std::os::unix::thread::RawPthread;
