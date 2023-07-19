use {
    fmod::{raw::*, *},
    smart_default::SmartDefault,
    std::{
        borrow::Cow,
        ffi::{c_char, CStr},
        mem, ptr, slice,
    },
};

/// # Setup.
impl System {
    /// Sets the maximum number of software mixed channels possible.
    ///
    /// This function cannot be called after FMOD is already activated, it must
    /// be called before [System::init], or after [System::close].
    ///
    /// <dl>
    /// <dt>Default</dt><dd>64</dd>
    /// </dl>
    pub fn set_software_channels(&self, num_software_channels: i32) -> Result {
        ffi!(FMOD_System_SetSoftwareChannels(
            self.as_raw(),
            num_software_channels,
        ))?;
        Ok(())
    }

    /// Retrieves the maximum number of software mixed channels possible.
    pub fn get_software_channels(&self) -> Result<i32> {
        let mut num_software_channels = 0;
        ffi!(FMOD_System_GetSoftwareChannels(
            self.as_raw(),
            &mut num_software_channels,
        ))?;
        Ok(num_software_channels)
    }

    /// Sets the output format for the software mixer.
    ///
    /// If loading Studio banks, this must be called with `speaker_mode`
    /// corresponding to the project output format if there is a possibility of
    /// the output audio device not matching the project format. Any differences
    /// between the project format and speakermode will cause the mix to sound
    /// wrong.
    ///
    /// By default `speaker_mode` will assume the setup the OS/output prefers.
    ///
    /// Altering the `sample_rate` from the OS/output preferred rate may incur
    /// extra latency. Altering the `speaker_mode` from the OS/output preferred
    /// mode may cause an upmix/downmix which can alter the sound.
    ///
    /// On lower power platforms such as mobile samplerate will default to 24KHz
    /// to reduce CPU cost.
    ///
    /// This function must be called before [System::init], or after
    /// [System::close].
    pub fn set_software_format(&self, format: SoftwareFormat) -> Result {
        let SoftwareFormat {
            sample_rate,
            speaker_mode,
            num_raw_speakers,
        } = format;
        ffi!(FMOD_System_SetSoftwareFormat(
            self.as_raw(),
            sample_rate,
            speaker_mode.into_raw(),
            num_raw_speakers,
        ))?;
        Ok(())
    }

    /// Retrieves the output format for the software mixer.
    pub fn get_software_format(&self) -> Result<SoftwareFormat> {
        let mut sample_rate = 0;
        let mut speaker_mode = SpeakerMode::default();
        let mut num_raw_speakers = 0;
        ffi!(FMOD_System_GetSoftwareFormat(
            self.as_raw(),
            &mut sample_rate,
            speaker_mode.as_raw_mut(),
            &mut num_raw_speakers
        ))?;
        Ok(SoftwareFormat {
            sample_rate,
            speaker_mode,
            num_raw_speakers,
        })
    }

    /// Sets the buffer size for the FMOD software mixing engine.
    ///
    /// This function is used if you need to control mixer latency or
    /// granularity. Smaller buffersizes lead to smaller latency, but can lead
    /// to stuttering/skipping/unstable sound on slower machines or soundcards
    /// with bad drivers.
    ///
    /// To get the `buffer_length` in milliseconds, divide it by the output rate
    /// and multiply the result by 1000. For a `buffer_length` of 1024 and an
    /// output rate of 48khz (see [System::set_software_format]), milliseconds =
    /// 1024 / 48000 * 1000 = 21.33ms. This means the mixer updates every
    /// 21.33ms.
    ///
    /// To get the total buffer size multiply the `buffer_length` by the
    /// `num_buffers` value. By default this would be 41024 = 4096 samples, or
    /// 421.33ms = 85.33ms. This would generally be the total latency of the
    /// software mixer, but in reality due to one of the buffers being written
    /// to constantly, and the cursor position of the buffer that is audible,
    /// the latency is typically more like the (number of buffers - 1.5)
    /// multiplied by the buffer length.
    ///
    /// To convert from milliseconds back to 'samples', simply multiply the
    /// value in milliseconds by the sample rate of the output (ie 48000 if that
    /// is what it is set to), then divide by 1000.
    ///
    /// The FMOD software mixer mixes to a ringbuffer. The size of this
    /// ringbuffer is determined here. It mixes a block of sound data every
    /// 'bufferlength' number of samples, and there are 'numbuffers' number of
    /// these blocks that make up the entire ringbuffer. Adjusting these values
    /// can lead to extremely low latency performance (smaller values), or
    /// greater stability in sound output (larger values).
    ///
    /// Warning! The 'buffersize' is generally best left alone. Making the
    /// granularity smaller will just increase CPU usage (cache misses and DSP
    /// network overhead). Making it larger affects how often you hear commands
    /// update such as volume/pitch/pan changes. Anything above 20ms will be
    /// noticeable and sound parameter changes will be obvious instead of
    /// smooth.
    ///
    /// FMOD chooses the most optimal size by default for best stability,
    /// depending on the output type. It is not recommended changing this value
    /// unless you really need to. You may get worse performance than the
    /// default settings chosen by FMOD. If you do set the size manually, the
    /// bufferlength argument must be a multiple of four, typically 256, 480,
    /// 512, 1024 or 2048 depedning on your latency requirements.
    ///
    /// The values in milliseconds and average latency expected from the
    /// settings can be calculated using the following code:
    ///
    /// ```rust,ignore
    /// let DspBufferSize { buffer_length, num_buffers } = system.get_dsp_buffer_size()?;
    /// let SoftwareFormat { sample_rate, .. } = system.get_software_format()?;
    ///
    /// let ms = buffer_size.buffer_length as f32 * 1000.0 / software_format.sample_rate as f32;
    ///
    /// println!("Mixer blocksize        = {:.02}", ms);
    /// println!("Mixer Total buffersize = {:.02}", ms * num_buffers);
    /// println!("Mixer Average Latency  = {:.02}", ms * (num_buffers as f32 - 1.5));
    /// ```
    pub fn set_dsp_buffer_size(&self, buffer_size: DspBufferSize) -> Result {
        let DspBufferSize {
            buffer_length,
            num_buffers,
        } = buffer_size;
        ffi!(FMOD_System_SetDSPBufferSize(
            self.as_raw(),
            buffer_length,
            num_buffers,
        ))?;
        Ok(())
    }

    /// Retrieves the buffer size settings for the FMOD software mixing engine.
    ///
    /// To get the `buffer_length` in milliseconds, divide it by the output rate
    /// and multiply the result by 1000. For a `buffer_length` of 1024 and an
    /// output rate of 48khz (see [System::set_software_format]), milliseconds =
    /// 1024 / 48000 * 1000 = 21.33ms. This means the mixer updates every
    /// 21.33ms.
    ///
    /// To get the total buffer size multiply the `buffer_length` by the
    /// `num_buffers` value. By default this would be 41024 = 4096 samples, or
    /// 421.33ms = 85.33ms. This would generally be the total latency of the
    /// software mixer, but in reality due to one of the buffers being written
    /// to constantly, and the cursor position of the buffer that is audible,
    /// the latency is typically more like the (number of buffers - 1.5)
    /// multiplied by the buffer length.
    ///
    /// To convert from milliseconds back to 'samples', simply multiply the
    /// value in milliseconds by the sample rate of the output (ie 48000 if that
    /// is what it is set to), then divide by 1000.
    pub fn get_dsp_buffer_size(&self) -> Result<(u32, i32)> {
        let mut bufferlength = 0;
        let mut numbuffers = 0;
        ffi!(FMOD_System_GetDSPBufferSize(
            self.as_raw(),
            &mut bufferlength,
            &mut numbuffers,
        ))?;
        Ok((bufferlength, numbuffers))
    }

    /// Sets the default file buffer size for newly opened streams.
    ///
    /// Valid units are [TimeUnit::Ms], [Pcm](TimeUnit::Pcm),
    /// [PcmBytes](TimeUnit::PcmBytes), and [RawBytes](TimeUnit::RawBytes).
    ///
    /// The default value is 16384 [TimeUnit::RawBytes]. Larger values will
    /// consume more memory, whereas smaller values may cause buffer under-run /
    /// starvation / stuttering caused by large delays in disk access (ie
    /// netstream), or CPU usage in slow machines, or by trying to play too many
    /// streams at once.
    ///
    /// Does not affect streams created with [Mode::OpenUser], as the buffer
    /// size is specified in [System::create_sound_ex].
    ///
    /// Does not affect latency of playback. All streams are pre-buffered
    /// (unless opened with [Mode::OpenOnly]), so they will always start
    /// immediately.
    ///
    /// Seek and Play operations can sometimes cause a reflush of this buffer.
    ///
    /// If [TimeUnit::RawBytes] is used, the memory allocated is two times the
    /// size passed in, because fmod allocates a double buffer.
    ///
    /// If [TimeUnit::Ms], [TimeUnit::Pcm] or [TimeUnit::PcmBytes] is used, and
    /// the stream is infinite (such as a shoutcast netstream), or VBR, then
    /// FMOD cannot calculate an accurate compression ratio to work with when
    /// the file is opened. This means it will then base the buffersize on
    /// [TimeUnit::PcmBytes], or in other words the number of PCM bytes, but
    /// this will be incorrect for some compressed formats. Use
    /// [TimeUnit::RawBytes] for these type (infinite / undetermined length) of
    /// streams for more accurate read sizes.
    ///
    /// To determine the actual memory usage of a stream, including sound buffer
    /// and other overhead, use [memory::get_stats] before and after creating a
    /// sound.
    ///
    /// Stream may still stutter if the codec uses a large amount of cpu time,
    /// which impacts the smaller, internal 'decode' buffer. The decode buffer
    /// size is changeable via [CreateSoundExInfo].
    pub fn set_stream_buffer_size(&self, file_buffer_size: Time) -> Result {
        ffi!(FMOD_System_SetStreamBufferSize(
            self.as_raw(),
            file_buffer_size.value,
            file_buffer_size.unit.into_raw(),
        ))?;
        Ok(())
    }

    /// Retrieves the default file buffer size for newly opened streams.
    ///
    /// Valid units are [TimeUnit::Ms], [Pcm](TimeUnit::Pcm),
    /// [PcmBytes](TimeUnit::PcmBytes), and [RawBytes](TimeUnit::RawBytes).
    pub fn get_stream_buffer_size(&self) -> Result<(u32, TimeUnit)> {
        let mut file_buffer_size = 0;
        let mut file_buffer_size_type = TimeUnit::zeroed();
        ffi!(FMOD_System_GetStreamBufferSize(
            self.as_raw(),
            &mut file_buffer_size,
            file_buffer_size_type.as_raw_mut(),
        ))?;
        Ok((file_buffer_size, file_buffer_size_type))
    }

    /// Sets advanced settings for the system object, typically to allow
    /// adjusting of settings related to resource usage or audio quality.
    pub fn set_advanced_settings(&self, mut advanced_settings: AdvancedSettings) -> Result {
        ffi!(FMOD_System_SetAdvancedSettings(
            self.as_raw(),
            advanced_settings.as_raw_mut(),
        ))?;
        Ok(())
    }

    /// Retrieves the advanced settings for the system object.
    pub fn get_advanced_settings(&self) -> Result<AdvancedSettings> {
        let mut advanced_settings = AdvancedSettings::default();
        ffi!(FMOD_System_GetAdvancedSettings(
            self.as_raw(),
            advanced_settings.as_raw_mut(),
        ))?;
        Ok(advanced_settings)
    }

    /// Sets the position of the specified speaker for the current speaker mode.
    ///
    /// This function allows the user to specify the position of their speaker
    /// to account for non standard setups.  
    /// It also allows the user to disable speakers from 3D consideration in a
    /// game.
    pub fn set_speaker_position(&self, speaker: Speaker, position: SpeakerPosition) -> Result {
        let SpeakerPosition { x, y, active } = position;
        ffi!(FMOD_System_SetSpeakerPosition(
            self.as_raw(),
            speaker.into_raw(),
            x,
            y,
            if active { 1 } else { 0 },
        ))?;
        Ok(())
    }

    /// Retrieves the position of the specified speaker for the current speaker
    /// mode.
    pub fn get_speaker_position(&self, speaker: Speaker) -> Result<SpeakerPosition> {
        let mut speaker_position = SpeakerPosition::default();
        let mut active = 0;
        ffi!(FMOD_System_GetSpeakerPosition(
            self.as_raw(),
            speaker.into_raw(),
            &mut speaker_position.x,
            &mut speaker_position.y,
            &mut active,
        ))?;
        speaker_position.active = active != 1;
        Ok(speaker_position)
    }

    /// Sets the global doppler scale, distance factor and log rolloff scale for
    /// all 3D sound in FMOD.
    ///
    /// See [Settings3d] for a description of what specificially this changes.
    pub fn set_3d_settings(&self, settings: Settings3d) -> Result {
        let Settings3d {
            doppler_scale,
            distance_factor,
            rolloff_scale,
        } = settings;
        ffi!(FMOD_System_Set3DSettings(
            self.as_raw(),
            doppler_scale,
            distance_factor,
            rolloff_scale,
        ))?;
        Ok(())
    }

    /// Retrieves the global doppler scale, distance factor and rolloff scale for all 3D sounds.
    pub fn get_3d_settings(&self) -> Result<Settings3d> {
        let mut settings = Settings3d::default();
        ffi!(FMOD_System_Get3DSettings(
            self.as_raw(),
            &mut settings.doppler_scale,
            &mut settings.distance_factor,
            &mut settings.rolloff_scale,
        ))?;
        Ok(settings)
    }

    /// Sets the number of 3D 'listeners' in the 3D sound scene.
    ///
    /// This function is useful mainly for split-screen game purposes.
    ///
    /// If the number of listeners is set to more than 1, then panning and
    /// doppler are turned off. All sound effects will be mono. FMOD uses a
    /// 'closest sound to the listener' method to determine what should be heard
    /// in this case.
    ///
    /// Users of the Studio API should call [studio::System::set_num_listeners]
    /// instead of this function.
    pub fn set_3d_num_listeners(&self, num_listeners: i32) -> Result {
        ffi!(FMOD_System_Set3DNumListeners(self.as_raw(), num_listeners))?;
        Ok(())
    }

    /// Retrieves the number of 3D listeners.
    ///
    /// Users of the Studio API should call [studio::System::get_num_listeners]
    /// instead of this function.
    pub fn get_3d_num_listeners(&self) -> Result<i32> {
        let mut num_listeners = 0;
        ffi!(FMOD_System_Get3DNumListeners(
            self.as_raw(),
            &mut num_listeners,
        ))?;
        Ok(num_listeners)
    }

    /// Sets a callback to allow custom calculation of distance attenuation.
    ///
    /// This function overrides [Mode::InverseRolloff3d],
    /// [Mode::LinearRolloff3d], [Mode::LinearSquareRolloff3d],
    /// [Mode::InverseTaperedRolloff3d], and [Mode::CustomRolloff3d].
    ///
    /// Set to `None` to return control of distance attenuation to FMOD.
    pub fn set_3d_rolloff_callback(&self, callback: Option<Rolloff3dCallback>) -> Result {
        ffi!(FMOD_System_Set3DRolloffCallback(
            self.as_raw(),
            mem::transmute(callback),
        ))?;
        Ok(())
    }
}

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
        #[deprecated]
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

/// Callback to allow custom calculation of distance attenuation.
pub type Rolloff3dCallback = extern "system" fn(channel: &Channel, distance: f32) -> f32;

enum_struct! {
    /// List of interpolation types used for resampling.
    ///
    /// Use [System::set_advanced_settings] and [AdvancedSettings::resampler_method] to configure the resampling quality you require for sample rate conversion during sound playback.
    pub enum DspResampler: FMOD_DSP_RESAMPLER {
        #[default]
        /// Default interpolation method, same as [DspResampler::Linear].
        Default  = FMOD_DSP_RESAMPLER_DEFAULT,
        /// No interpolation. High frequency aliasing hiss will be audible depending on the sample rate of the sound.
        NoInterp = FMOD_DSP_RESAMPLER_NOINTERP,
        /// Linear interpolation (default method). Fast and good quality, causes very slight lowpass effect on low frequency sounds.
        Linear   = FMOD_DSP_RESAMPLER_LINEAR,
        /// Cubic interpolation. Slower than linear interpolation but better quality.
        Cubic    = FMOD_DSP_RESAMPLER_CUBIC,
        /// 5 point spline interpolation. Slowest resampling method but best quality.
        Spline   = FMOD_DSP_RESAMPLER_SPLINE,
    }
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
