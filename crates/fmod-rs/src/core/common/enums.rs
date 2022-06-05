use fmod::{raw::*, *};

enum_struct! {
    /// Flags that provide additional information about a particular driver.
    pub enum DriverState: u32 {
        /// Device is currently plugged in.
        Connected = FMOD_DRIVER_STATE_CONNECTED,
        #[default]
        /// Device is the users preferred choice.
        Default   = FMOD_DRIVER_STATE_DEFAULT,
    }

    /// Time types used for position or length.
    pub enum TimeUnit: u32 {
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

    /// Scheduling priority to assign a given thread to.
    ///
    /// The platform agnostic priorities are used to rank FMOD threads against one another for best runtime scheduling.
    /// Platforms will translate these values in to platform specific priorities.
    /// See the FMOD platform specific docs for each platform to see how the agnostic priorities map to specific values.
    ///
    /// Explicit platform specific priorities can be given within the range of [ThreadPriority::PlatformMin] to [ThreadPriority::PlatformMax].
    /// See platform documentation for details on the available priority values for a given operating system.
    pub enum ThreadPriority: i32 {
        // Platform specific priority range
        /// Lower bound of platform specific priority range.
        PlatformMin = FMOD_THREAD_PRIORITY_PLATFORM_MIN,
        /// Upper bound of platform specific priority range.
        PlatformMax = FMOD_THREAD_PRIORITY_PLATFORM_MAX as i32,
        // Platform agnostic priorities, maps internally to platform specific value
        #[default]
        /// For a given thread use the default listed below, i.e. [ThreadType::Mixer] uses [ThreadPriority::Mixer].
        Default  = FMOD_THREAD_PRIORITY_DEFAULT,
        /// Low platform agnostic priority.
        Low      = FMOD_THREAD_PRIORITY_LOW,
        /// Medium platform agnostic priority.
        Medium   = FMOD_THREAD_PRIORITY_MEDIUM,
        /// High platform agnostic priority.
        High     = FMOD_THREAD_PRIORITY_HIGH,
        /// Very high platform agnostic priority.
        VeryHigh = FMOD_THREAD_PRIORITY_VERY_HIGH,
        /// Extreme platform agnostic priority.
        Extreme  = FMOD_THREAD_PRIORITY_EXTREME,
        /// Critical platform agnostic priority.
        Critical = FMOD_THREAD_PRIORITY_CRITICAL,
        // Thread defaults
        /// Default priority for [ThreadType::Mixer].
        Mixer            = FMOD_THREAD_PRIORITY_MIXER,
        /// Default priority for [ThreadType::Feeder].
        Feeder           = FMOD_THREAD_PRIORITY_FEEDER,
        /// Default priority for [ThreadType::Stream].
        Stream           = FMOD_THREAD_PRIORITY_STREAM,
        /// Default priority for [ThreadType::File].
        File             = FMOD_THREAD_PRIORITY_FILE,
        /// Default priority for [ThreadType::NonBlocking].
        NonBlocking      = FMOD_THREAD_PRIORITY_NONBLOCKING,
        /// Default priority for [ThreadType::Record].
        Record           = FMOD_THREAD_PRIORITY_RECORD,
        /// Default priority for [ThreadType::Geometry].
        Geometry         = FMOD_THREAD_PRIORITY_GEOMETRY,
        /// Default priority for [ThreadType::Profiler].
        Profiler         = FMOD_THREAD_PRIORITY_PROFILER,
        /// Default priority for [ThreadType::StudioUpdate].
        StudioUpdate     = FMOD_THREAD_PRIORITY_STUDIO_UPDATE,
        /// Default priority for [ThreadType::StudioLoadBank].
        StudioLoadBank   = FMOD_THREAD_PRIORITY_STUDIO_LOAD_BANK,
        /// Default priority for [ThreadType::StudioLoadSample].
        StudioLoadSample = FMOD_THREAD_PRIORITY_STUDIO_LOAD_SAMPLE,
        /// Default priority for [ThreadType::Convolution1].
        Convolution1     = FMOD_THREAD_PRIORITY_CONVOLUTION1,
        /// Default priority for [ThreadType::Convolution2].
        Convolution2     = FMOD_THREAD_PRIORITY_CONVOLUTION2,
    }

    /// Stack space available to the given thread.
    ///
    /// Stack size can be specified explicitly, however for each thread you should provide a size equal to or larger than the expected default or risk causing a stack overflow at runtime.
    pub enum ThreadStackSize: u32 {
        #[default]
        /// For a given thread use the default listed below, i.e. [ThreadType::Mixer] uses [ThreadStackSize::Mixer].
        Default          = FMOD_THREAD_STACK_SIZE_DEFAULT,
        /// Default stack size for [ThreadType::Mixer].
        Mixer            = FMOD_THREAD_STACK_SIZE_MIXER,
        /// Default stack size for [ThreadType::Feeder].
        Feeder           = FMOD_THREAD_STACK_SIZE_FEEDER,
        /// Default stack size for [ThreadType::Stream].
        Stream           = FMOD_THREAD_STACK_SIZE_STREAM,
        /// Default stack size for [ThreadType::File].
        File             = FMOD_THREAD_STACK_SIZE_FILE,
        /// Default stack size for [ThreadType::NonBlocking].
        NonBlocking      = FMOD_THREAD_STACK_SIZE_NONBLOCKING,
        /// Default stack size for [ThreadType::Record].
        Record           = FMOD_THREAD_STACK_SIZE_RECORD,
        /// Default stack size for [ThreadType::Geometry].
        Geometry         = FMOD_THREAD_STACK_SIZE_GEOMETRY,
        /// Default stack size for [ThreadType::Profiler].
        Profiler         = FMOD_THREAD_STACK_SIZE_PROFILER,
        /// Default stack size for [ThreadType::StudioUpdate].
        StudioUpdate     = FMOD_THREAD_STACK_SIZE_STUDIO_UPDATE,
        /// Default stack size for [ThreadType::StudioLoadBank].
        StudioLoadBank   = FMOD_THREAD_STACK_SIZE_STUDIO_LOAD_BANK,
        /// Default stack size for [ThreadType::StudioLoadSample].
        StudioLoadSample = FMOD_THREAD_STACK_SIZE_STUDIO_LOAD_SAMPLE,
        /// Default stack size for [ThreadType::Convolution1].
        Convolution1     = FMOD_THREAD_STACK_SIZE_CONVOLUTION1,
        /// Default stack size for [ThreadType::Convolution2].
        Convolution2     = FMOD_THREAD_STACK_SIZE_CONVOLUTION2,
    }

    /// Named constants for threads created at runtime.
    pub enum ThreadType: i32 {
        /// Thread responsible for mixing and processing blocks of audio.
        Mixer            = FMOD_THREAD_TYPE_MIXER,
        /// Thread used by some output plugins for transferring buffered audio from [ThreadType::Mixer] to the sound output device.
        Feeder           = FMOD_THREAD_TYPE_FEEDER,
        /// Thread that decodes compressed audio to PCM for Sounds created as [Mode::CreateStream].
        Stream           = FMOD_THREAD_TYPE_STREAM,
        /// Thread that reads compressed audio from disk to be consumed by [ThreadType::Stream].
        File             = FMOD_THREAD_TYPE_FILE,
        /// Thread that processes the creation of Sounds asynchronously when opened with [Mode::NonBlocking].
        NonBlocking      = FMOD_THREAD_TYPE_NONBLOCKING,
        /// Thread used by some output plugins for transferring audio from a microphone to [ThreadType::Mixer].
        Record           = FMOD_THREAD_TYPE_RECORD,
        /// Thread used by the [Geometry] system for performing background calculations.
        Geometry         = FMOD_THREAD_TYPE_GEOMETRY,
        /// Thread for network communication when using [InitFlags::ProfileEnable].
        Profiler         = FMOD_THREAD_TYPE_PROFILER,
        /// Thread for processing Studio API commands and scheduling sound playback.
        StudioUpdate     = FMOD_THREAD_TYPE_STUDIO_UPDATE,
        /// Thread for asynchronously loading [studio::Bank] metadata.
        StudioLoadBank   = FMOD_THREAD_TYPE_STUDIO_LOAD_BANK,
        /// Thread for asynchronously loading [studio::Bank] sample data.
        StudioLoadSample = FMOD_THREAD_TYPE_STUDIO_LOAD_SAMPLE,
        /// Thread for processing medium size delay lines for [DspType::ConvolutionReverb].
        Convolution1     = FMOD_THREAD_TYPE_CONVOLUTION1,
        /// Thread for processing larger size delay lines for [DspType::ConvolutionReverb].
        Convolution2     = FMOD_THREAD_TYPE_CONVOLUTION2,
    }

    /// Identifier used to distinguish between Channel and ChannelGroup in the ChannelControl callback.
    pub enum ChannelControlType: i32 {
        /// Type representing [Channel]
        Channel      = FMOD_CHANNELCONTROL_CHANNEL,
        /// Type representing [ChannelGroup]
        ChannelGroup = FMOD_CHANNELCONTROL_CHANNELGROUP,
    }

    /// Built-in output types that can be used to run the mixer.
    ///
    /// To pass information to the driver when initializing use the `extra_driver_data` parameter in [System::init_ex] for the following reasons:
    ///
    /// - [OutputType::WavWriter] - `*const c_char` file name that the wav writer will output to.
    /// - [OutputType::WavWriterNrt] - `*const c_char` file name that the wav writer will output to.
    /// - [OutputType::PulseAudio] - `*const c_char` application name to display in OS audio mixer.
    /// - [OutputType::Asio] - `*mut c_void` application window handle.
    ///
    /// Currently these are the only FMOD drivers that take extra information. Other unknown plugins may have different requirements.
    ///
    /// If [OutputType::WavWriterNrt] or [OutputType::NoSoundNrt] are used, and if the [System::update] function is being called very quickly (ie for a non realtime decode) it may be being called too quickly for the FMOD streamer thread to respond to. The result will be a skipping/stuttering output in the captured audio. To remedy this, disable the FMOD streamer thread, and use [InitFlags::StreamFromUpdate] to avoid skipping in the output stream, as it will lock the mixer and the streamer together in the same thread.
    pub enum OutputType: i32 {
        /// Picks the best output mode for the platform. This is the default.
        AutoDetect   = FMOD_OUTPUTTYPE_AUTODETECT,
        /// All - 3rd party plugin, unknown. This is for use with [System::get_output] only.
        Unknown      = FMOD_OUTPUTTYPE_UNKNOWN,
        /// All - Perform all mixing but discard the final output.
        NoSound      = FMOD_OUTPUTTYPE_NOSOUND,
        /// All - Writes output to a .wav file.
        WavWriter    = FMOD_OUTPUTTYPE_WAVWRITER,
        /// All - Non-realtime version of [OutputType::NoSound], one mix per [System::update].
        NoSoundNrt   = FMOD_OUTPUTTYPE_NOSOUND_NRT,
        /// All - Non-realtime version of [OutputType::WavWriter], one mix per [System::update].
        WavWriterNrt = FMOD_OUTPUTTYPE_WAVWRITER_NRT,
        /// Win / UWP / Xbox One / Game Core - Windows Audio Session API. (Default on Windows, Xbox One, Game Core and UWP)
        Wasapi       = FMOD_OUTPUTTYPE_WASAPI,
        /// Win - Low latency ASIO 2.0.
        Asio         = FMOD_OUTPUTTYPE_ASIO,
        /// Linux - Pulse Audio. (Default on Linux if available)
        PulseAudio   = FMOD_OUTPUTTYPE_PULSEAUDIO,
        /// Linux - Advanced Linux Sound Architecture. (Default on Linux if PulseAudio isn't available)
        Alsa         = FMOD_OUTPUTTYPE_ALSA,
        /// Mac / iOS - Core Audio. (Default on Mac and iOS)
        CoreAudio    = FMOD_OUTPUTTYPE_COREAUDIO,
        /// Android - Java Audio Track. (Default on Android 2.2 and below)
        AudioTrack   = FMOD_OUTPUTTYPE_AUDIOTRACK,
        /// Android - OpenSL ES. (Default on Android 2.3 up to 7.1)
        OpenSl       = FMOD_OUTPUTTYPE_OPENSL,
        /// PS4 / PS5 - Audio Out. (Default on PS4, PS5)
        AudioOut     = FMOD_OUTPUTTYPE_AUDIOOUT,
        /// PS4 - Audio3D.
        Audio3d      = FMOD_OUTPUTTYPE_AUDIO3D,
        /// HTML5 - Web Audio ScriptProcessorNode output. (Default on HTML5 if AudioWorkletNode isn't available)
        WebAudio     = FMOD_OUTPUTTYPE_WEBAUDIO,
        /// Switch - nn::audio. (Default on Switch)
        NnAudio      = FMOD_OUTPUTTYPE_NNAUDIO,
        /// Win10 / Xbox One / Game Core - Windows Sonic.
        Winsonic     = FMOD_OUTPUTTYPE_WINSONIC,
        /// Android - AAudio. (Default on Android 8.1 and above)
        AAudio       = FMOD_OUTPUTTYPE_AAUDIO,
        /// HTML5 - Web Audio AudioWorkletNode output. (Default on HTML5 if available)
        AudioWorklet = FMOD_OUTPUTTYPE_AUDIOWORKLET,
    }
}

raw! {
    enum_struct! {
        /// Specify the destination of log output when using the logging version of FMOD.
        ///
        /// TTY destination can vary depending on platform, common examples include the Visual Studio / Xcode output window, stderr and LogCat.
        pub enum DebugMode: i32 {
            /// Default log location per platform, i.e. Visual Studio output window, stderr, LogCat, etc.
            Tty      = FMOD_DEBUG_MODE_TTY,
            /// Write log to specified file path.
            File     = FMOD_DEBUG_MODE_FILE,
            /// Call specified callback with log information.
            Callback = FMOD_DEBUG_MODE_CALLBACK,
        }
    }
}

enum_struct! {
    /// Speaker mode types.
    ///
    /// Note below the phrase 'sound channels' is used. These are the subchannels inside a sound, they are not related and have nothing to do with the FMOD class "Channel".
    ///
    /// For example a mono sound has 1 sound channel, a stereo sound has 2 sound channels, and an AC3 or 6 channel wav file have 6 "sound channels".
    ///
    /// See the FMOD Studio Mixing Guide for graphical depictions of each speaker mode.
    pub enum SpeakerMode: i32 {
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

    /// Assigns an enumeration for a speaker index.
    pub enum Speaker: i32 {
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

    /// Speaker ordering for multichannel signals.
    pub enum ChannelOrder: i32 {
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

    /// Types of plugin used to extend functionality.
    pub enum PluginType: i32 {
        /// Audio output interface plugin represented with [OutputDescription].
        Output = FMOD_PLUGINTYPE_OUTPUT,
        /// File format codec plugin represented with [CodecDescription].
        Codec  = FMOD_PLUGINTYPE_CODEC,
        /// DSP unit plugin represented with [DspDescription].
        Dsp    = FMOD_PLUGINTYPE_DSP,
    }

    /// Recognized audio formats that can be loaded into a Sound.
    pub enum SoundType: i32 {
        /// Unknown or custom codec plugin.
        Unknown         = FMOD_SOUND_TYPE_UNKNOWN,
        /// Audio Interchange File Format (.aif, .aiff). Uncompressed integer formats only.
        Aiff            = FMOD_SOUND_TYPE_AIFF,
        /// Microsoft Advanced Systems Format (.asf, .wma, .wmv). Platform provided decoder, available only on Windows.
        Asf             = FMOD_SOUND_TYPE_ASF,
        /// Downloadable Sound (.dls). Multi-sound bank format used by MIDI (.mid).
        Dls             = FMOD_SOUND_TYPE_DLS,
        /// Free Lossless Audio Codec (.flac).
        Flac            = FMOD_SOUND_TYPE_FLAC,
        /// FMOD Sample Bank (.fsb). Proprietary multi-sound bank format. Supported encodings: PCM16, FADPCM, Vorbis, AT9, XMA, Opus.
        Fsb             = FMOD_SOUND_TYPE_FSB,
        /// Impulse Tracker (.it).
        It              = FMOD_SOUND_TYPE_IT,
        /// Musical Instrument Digital Interface (.mid).
        Midi            = FMOD_SOUND_TYPE_MIDI,
        /// Protracker / Fasttracker Module File (.mod).
        Mod             = FMOD_SOUND_TYPE_MOD,
        /// Moving Picture Experts Group (.mp2, .mp3). Also supports .wav (RIFF) container format.
        Mpeg            = FMOD_SOUND_TYPE_MPEG,
        /// Ogg Vorbis (.ogg).
        OggVorbis       = FMOD_SOUND_TYPE_OGGVORBIS,
        /// Play list information container (.asx, .pls, .m3u, .wax). No audio, tags only.
        Playlist        = FMOD_SOUND_TYPE_PLAYLIST,
        /// Raw uncompressed PCM data (.raw).
        Raw             = FMOD_SOUND_TYPE_RAW,
        /// ScreamTracker 3 Module (.s3m).
        S3m             = FMOD_SOUND_TYPE_S3M,
        /// User created sound.
        User            = FMOD_SOUND_TYPE_USER,
        /// Microsoft Waveform Audio File Format (.wav). Supported encodings: Uncompressed PCM, IMA ADPCM. Platform provided ACM decoder extensions, available only on Windows.
        Wav             = FMOD_SOUND_TYPE_WAV,
        /// FastTracker 2 Extended Module (.xm).
        Xm              = FMOD_SOUND_TYPE_XM,
        /// Microsoft XMA bit-stream supported by FSB (.fsb) container format. Platform provided decoder, available only on Xbox.
        Xma             = FMOD_SOUND_TYPE_XMA,
        /// Apple Audio Queue decoder (.mp4, .m4a, .mp3). Supported encodings: AAC, ALAC, MP3. Platform provided decoder, available only on iOS / tvOS devices.
        AudioQueue      = FMOD_SOUND_TYPE_AUDIOQUEUE,
        /// Sony ATRAC9 bit-stream supported by FSB (.fsb) container format. Platform provided decoder, available only on PlayStation.
        At9             = FMOD_SOUND_TYPE_AT9,
        /// Vorbis bit-stream supported by FSB (.fsb) container format.
        Vorbis          = FMOD_SOUND_TYPE_VORBIS,
        /// Microsoft Media Foundation decoder (.asf, .wma, .wmv, .mp4, .m4a). Platform provided decoder, available only on UWP.
        MediaFoundation = FMOD_SOUND_TYPE_MEDIA_FOUNDATION,
        /// Google Media Codec decoder (.m4a, .mp4). Platform provided decoder, available only on Android.
        MediaCodec      = FMOD_SOUND_TYPE_MEDIACODEC,
        /// FMOD Adaptive Differential Pulse Code Modulation bit-stream supported by FSB (.fsb) container format.
        Fadpcm          = FMOD_SOUND_TYPE_FADPCM,
        /// Opus bit-stream supported by FSB (.fsb) container format. Platform provided decoder, available only on Xbox Series X|S.
        Opus            = FMOD_SOUND_TYPE_OPUS,
    }

    /// These definitions describe the native format of the hardware or software buffer that will be used.
    pub enum SoundFormat: i32 {
        /// Uninitalized / unknown.
        None      = FMOD_SOUND_FORMAT_NONE,
        /// 8bit integer PCM data.
        Pcm8      = FMOD_SOUND_FORMAT_PCM8,
        /// 16bit integer PCM data.
        Pcm16     = FMOD_SOUND_FORMAT_PCM16,
        /// 24bit integer PCM data.
        Pcm24     = FMOD_SOUND_FORMAT_PCM24,
        /// 32bit integer PCM data.
        Pcm32     = FMOD_SOUND_FORMAT_PCM32,
        /// 32bit floating point PCM data.
        PcmFloat  = FMOD_SOUND_FORMAT_PCMFLOAT,
        /// Sound data is in its native compressed format. See [Mode::CreateCompressedSample]
        Bitstream = FMOD_SOUND_FORMAT_BITSTREAM,
    }

    /// These values describe what state a sound is in after [Mode::NonBlocking] has been used to open it.
    ///
    /// With streams, if you are using [Mode::NonBlocking], note that if the user calls [Sound::get_sub_sound], a stream will go into [OpenState::Seeking] state and sound related commands will return [Error::NotReady].
    ///
    ///With streams, if you are using [Mode::NonBlocking], note that if the user calls [Channel::get_position], a stream will go into [OpenState::SetPosition] state and sound related commands will return [Error::NotReady].
    pub enum OpenState: i32 {
        /// Opened and ready to play.
        Ready       = FMOD_OPENSTATE_READY,
        /// Initial load in progress.
        Loading     = FMOD_OPENSTATE_LOADING,
        /// Failed to open - file not found, out of memory etc. See return value of [Sound::get_open_state] for what happened.
        Error       = FMOD_OPENSTATE_ERROR,
        /// Connecting to remote host (internet sounds only).
        Connecting  = FMOD_OPENSTATE_CONNECTING,
        /// Buffering data.
        Buffering   = FMOD_OPENSTATE_BUFFERING,
        /// Seeking to subsound and re-flushing stream buffer.
        Seeking     = FMOD_OPENSTATE_SEEKING,
        /// Ready and playing, but not possible to release at this time without stalling the main thread.
        Playing     = FMOD_OPENSTATE_PLAYING,
        /// Seeking within a stream to a different position.
        SetPosition = FMOD_OPENSTATE_SETPOSITION,
    }

    /// Values specifying behavior when a sound group's max audible value is exceeded.
    ///
    /// When using [SoundGroupBehavior::Mute], [SoundGroup::set_mute_fade_speed] can be used to stop a sudden transition.
    /// Instead, the time specified will be used to cross fade between the sounds that go silent and the ones that become audible.
    pub enum SoundGroupBehavior: i32 {
        /// Excess sounds will fail when calling [System::play_sound].
        Fail        = FMOD_SOUNDGROUP_BEHAVIOR_FAIL,
        /// Excess sounds will begin mute and will become audible when sufficient sounds are stopped.
        Mute        = FMOD_SOUNDGROUP_BEHAVIOR_MUTE,
        /// Excess sounds will steal from the quietest [Sound] playing in the group.
        StealLowest = FMOD_SOUNDGROUP_BEHAVIOR_STEALLOWEST,
    }

    /// Types of callbacks called by Channels and ChannelGroups.
    pub enum ChannelControlCallbackType: i32 {
        /// Called when a sound ends. Supported by [Channel] only.
        End          = FMOD_CHANNELCONTROL_CALLBACK_END,
        /// Called when a [Channel] is made virtual or real. Supported by [Channel] objects only.
        ///
        /// - `command_data_1`: (int) 0 represents 'virtual to real' and 1 represents 'real to virtual'.
        VirtualVoice = FMOD_CHANNELCONTROL_CALLBACK_VIRTUALVOICE,
        /// Called when a syncpoint is encountered. Can be from wav file markers or user added. Supported by [Channel] only.
        ///
        /// - `command_data_1`: (int) representing the index of the sync point for use with [Sound::get_sync_point_info].
        SyncPoint    = FMOD_CHANNELCONTROL_CALLBACK_SYNCPOINT,
        /// Called when geometry occlusion values are calculated. Can be used to clamp or change the value. Supported by [Channel] and [ChannelGroup].
        Occlusion    = FMOD_CHANNELCONTROL_CALLBACK_OCCLUSION,
    }

    /// References to built in DSP positions that reside in a Channel or ChannelGroup DSP chain.
    ///
    /// Before any [Dsp]s have been added by the user, there is only one [Dsp] available for a [Channel] or [ChannelGroup]. This is of type [DspType::Fader]. This handles volume and panning for a [Channel] or [ChannelGroup].
    /// As only 1 [Dsp] exists by default, initially [ChannelControlDspIndex::Head], [ChannelControlDspIndex::Tail] and [ChannelControlDspIndex::Fader] all reference the same DSP.
    pub enum ChannelControlDspIndex: i32 {
        /// Head of the DSP chain, equivalent of index 0.
        Head  = FMOD_CHANNELCONTROL_DSP_HEAD,
        /// Built in fader DSP.
        Fader = FMOD_CHANNELCONTROL_DSP_FADER,
        /// Tail of the DSP chain, equivalent of the number of [Dsp]s minus 1.
        Tail  = FMOD_CHANNELCONTROL_DSP_TAIL,
    }

    /// Identifier used to represent the different types of instance in the error callback.
    pub enum ErrorCallbackInstaceType: i32 {
        /// Type representing no known instance type.
        None                    = FMOD_ERRORCALLBACK_INSTANCETYPE_NONE,
        /// Type representing [System].
        System                  = FMOD_ERRORCALLBACK_INSTANCETYPE_SYSTEM,
        /// Type representing [Channel].
        Channel                 = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNEL,
        /// Type representing [ChannelGroup].
        ChannelGroup            = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNELGROUP,
        /// Type representing [ChannelControl].
        ChannelControl          = FMOD_ERRORCALLBACK_INSTANCETYPE_CHANNELCONTROL,
        /// Type representing [Sound].
        Sound                   = FMOD_ERRORCALLBACK_INSTANCETYPE_SOUND,
        /// Type representing [SoundGroup].
        SoundGroup              = FMOD_ERRORCALLBACK_INSTANCETYPE_SOUNDGROUP,
        /// Type representing [Dsp].
        Dsp                     = FMOD_ERRORCALLBACK_INSTANCETYPE_DSP,
        /// Type representing [DspConnection].
        DspConnection           = FMOD_ERRORCALLBACK_INSTANCETYPE_DSPCONNECTION,
        /// Type representing [Geometry].
        Geometry                = FMOD_ERRORCALLBACK_INSTANCETYPE_GEOMETRY,
        /// Type representing [Reverb3d].
        Reverb3d                = FMOD_ERRORCALLBACK_INSTANCETYPE_REVERB3D,
        /// Type representing [studio::System].
        StudioSystem            = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_SYSTEM,
        /// Type representing [studio::EventDescription].
        StudioEventDescription  = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_EVENTDESCRIPTION,
        /// Type representing [studio::EventInstance].
        StudioEventInstance     = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_EVENTINSTANCE,
        /// Deprecated.
        StudioParameterInstance = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_PARAMETERINSTANCE,
        /// Type representing [studio::Bus].
        StudioBus               = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_BUS,
        /// Type representing [studio::Vca].
        StudioVca               = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_VCA,
        /// Type representing [studio::Bank].
        StudioBank              = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_BANK,
        /// Type representing [studio::CommandReplay].
        StudioCommandReplay     = FMOD_ERRORCALLBACK_INSTANCETYPE_STUDIO_COMMANDREPLAY,
    }

    /// List of interpolation types used for resampling.
    ///
    /// Use [System::set_advanced_settings] and [AdvancedSettings::resampler_method] to configure the resampling quality you require for sample rate conversion during sound playback.
    pub enum DspResampler: i32 {
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

    /// List of connection types between 2 DSP nodes.
    pub enum DspConnectionType: i32 {
        #[default]
        /// Default connection type. Audio is mixed from the input to the output [Dsp]'s audible buffer.
        ///
        /// Default [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s audible buffer, meaning it will be part of the audible signal. A standard connection will execute its input [Dsp] if it has not been executed before.
        Standard      = FMOD_DSPCONNECTION_TYPE_STANDARD,
        /// Sidechain connection type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer.
        ///
        /// Sidechain [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, meaning it will NOT be part of the audible signal. A sidechain connection will execute its input [Dsp] if it has not been executed before.
        ///
        /// The purpose of the seperate sidechain buffer in a [Dsp], is so that the [Dsp] effect can privately access for analysis purposes. An example of use in this case, could be a compressor which analyzes the signal, to control its own effect parameters (ie a compression level or gain).
        ///
        /// For the effect developer, to accept sidechain data, the sidechain data will appear in the [DspState] struct which is passed into the read callback of a [Dsp] unit.
        ///
        /// [DspState::sidechain_data] and [DspState::sidechain_channels] will hold the mixed result of any sidechain data flowing into it.
        Sidechain     = FMOD_DSPCONNECTION_TYPE_SIDECHAIN,
        /// Send connection type. Audio is mixed from the input to the output [Dsp]'s audible buffer, but the input is NOT executed, only copied from. A standard connection or sidechain needs to make an input execute to generate data.
        ///
        /// Send [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s audible buffer, meaning it will be part of the audible signal. A send connection will NOT execute its input [Dsp] if it has not been executed before.
        ///
        /// A send connection will only read what exists at the input's buffer at the time of executing the output [Dsp] unit (which can be considered the 'return')
        Send          = FMOD_DSPCONNECTION_TYPE_SEND,
        /// Send sidechain connection type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, but the input is NOT executed, only copied from. A standard connection or sidechain needs to make an input execute to generate data.
        ///
        /// Send sidechain [DspConnection] type. Audio is mixed from the input to the output [Dsp]'s sidechain buffer, meaning it will NOT be part of the audible signal. A send sidechain connection will NOT execute its input [Dsp] if it has not been executed before.
        ///
        /// A send sidechain connection will only read what exists at the input's buffer at the time of executing the output [Dsp] unit (which can be considered the 'sidechain return').
        ///
        /// For the effect developer, to accept sidechain data, the sidechain data will appear in the [DspState] struct which is passed into the read callback of a [Dsp] unit.
        ///
        /// [DspState::sidechain_data] and [DspState::sidechain_channels] will hold the mixed result of any sidechain data flowing into it.
        SendSidechain = FMOD_DSPCONNECTION_TYPE_SEND_SIDECHAIN,
    }

    /// List of tag data / metadata types that could be stored within a sound. These include id3 tags, metadata from netstreams and vorbis/asf data.
    pub enum TagType: i32 {
        /// Tag type that is not recognized by FMOD
        Unknown       = FMOD_TAGTYPE_UNKNOWN,
        /// MP3 ID3 Tag 1.0. Typically 1 tag stored 128 bytes from end of an MP3 file.
        Id3v1         = FMOD_TAGTYPE_ID3V1,
        /// MP3 ID3 Tag 2.0. Variable length tags with more than 1 possible.
        Id3v2         = FMOD_TAGTYPE_ID3V2,
        /// Metadata container used in Vorbis, FLAC, Theora, Speex and Opus file formats.
        VorbisComment = FMOD_TAGTYPE_VORBISCOMMENT,
        /// SHOUTcast internet stream metadata which can be issued during playback.
        ShoutCast     = FMOD_TAGTYPE_SHOUTCAST,
        /// Icecast internet stream metadata which can be issued during playback.
        Icecast       = FMOD_TAGTYPE_ICECAST,
        /// Advanced Systems Format metadata typically associated with Windows Media formats such as WMA.
        Asf           = FMOD_TAGTYPE_ASF,
        /// Metadata stored inside a MIDI file.
        ///
        /// Remarks. A midi file contains 16 channels. Not all of them are used, or in order. Use the tag 'Channel mask' and 'Number of channels' to find the channels used, to use with [Sound::set_music_channel_volume] / [Sound::get_music_channel_volume]. For example if the mask is 1001b, there are 2 channels, and channel 0 and channel 3 are the 2 channels used with the above functions.
        Midi          = FMOD_TAGTYPE_MIDI,
        /// Playlist files such as PLS,M3U,ASX and WAX will populate playlist information through this tag type.
        Playlist      = FMOD_TAGTYPE_PLAYLIST,
        /// Tag type used by FMOD's MIDI, MOD, S3M, XM, IT format support, and netstreams to notify of internet stream events like a sample rate change.
        Fmod          = FMOD_TAGTYPE_FMOD,
        /// For codec developers, this tag type can be used with [CodecMetadataFunc] to generate custom metadata.
        User          = FMOD_TAGTYPE_USER,
    }
}

raw! {
    enum_struct! {
        /// List of tag data / metadata types.
        ///
        /// See [Tag] structure for tag length in bytes.
        pub enum TagDataType: i32 {
            /// Raw binary data. see [Tag] structure for length of data in bytes.
            Binary        = FMOD_TAGDATATYPE_BINARY,
            /// Integer - Note this integer could be 8bit / 16bit / 32bit / 64bit. See [Tag] structure for integer size (1 vs 2 vs 4 vs 8 bytes).
            Int           = FMOD_TAGDATATYPE_INT,
            /// IEEE floating point number. See [Tag] structure to confirm if the float data is 32bit or 64bit (4 vs 8 bytes).
            Float         = FMOD_TAGDATATYPE_FLOAT,
            /// 8bit ASCII char string. See [Tag] structure for string length in bytes.
            String        = FMOD_TAGDATATYPE_STRING,
            /// 16bit UTF string. Assume little endian byte order. See [Tag] structure for string length in bytes.
            StringUtf16   = FMOD_TAGDATATYPE_STRING_UTF16,
            /// 16bit UTF string Big endian byte order. See [Tag] structure for string length in bytes.
            StringUtf16be = FMOD_TAGDATATYPE_STRING_UTF16BE,
            /// 8 bit UTF string. See [Tag] structure for string length in bytes.
            StringUtf8    = FMOD_TAGDATATYPE_STRING_UTF8,
        }
    }
}

enum_struct! {
    /// Port types available for routing audio.
    pub enum PortType: i32 {
        Music          = FMOD_PORT_TYPE_MUSIC,
        CopyrightMusic = FMOD_PORT_TYPE_COPYRIGHT_MUSIC,
        Voice          = FMOD_PORT_TYPE_VOICE,
        Controller     = FMOD_PORT_TYPE_CONTROLLER,
        Personal       = FMOD_PORT_TYPE_PERSONAL,
        Vibration      = FMOD_PORT_TYPE_VIBRATION,
        Aux            = FMOD_PORT_TYPE_AUX,
    }

    #[cfg_attr(feature = "unstable", doc(cfg(target_os = "ios")))]
    /// Control whether the sound will use a the dedicated hardware decoder or a
    /// software codec.
    ///
    /// Every devices has a single hardware decoder and unlimited software
    /// decoders.
    pub enum AudioQueueCodecPolicy: i32 {
        /// Try hardware first, if it's in use or prohibited by audio session,
        /// try software.
        Default      = FMOD_AUDIOQUEUE_CODECPOLICY_DEFAULT,
        /// `kAudioQueueHardwareCodecPolicy_UseSoftwareOnly` ~ try software,
        /// if not available fail.
        SoftwareOnly = FMOD_AUDIOQUEUE_CODECPOLICY_SOFTWAREONLY,
        /// `kAudioQueueHardwareCodecPolicy_UseHardwareOnly` ~ try hardware,
        /// if not available fail.
        HardwareOnly = FMOD_AUDIOQUEUE_CODECPOLICY_HARDWAREONLY,
    }
}
