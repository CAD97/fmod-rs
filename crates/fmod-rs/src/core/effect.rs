use fmod::{raw::*, *};

enum_struct! {
    /// DSP types.
    pub enum DspType: i32 {
        /// Was created via a non-FMOD plugin and has an unknown purpose.
        Unknown           = FMOD_DSP_TYPE_UNKNOWN,
        /// Does not process the signal, acts as a unit purely for mixing inputs.
        Mixer             = FMOD_DSP_TYPE_MIXER,
        /// Generates sine/square/saw/triangle or noise tones. See
        /// [DspOscillator] for parameter information,
        /// [Effect reference - Oscillator] for overview.
        ///
        /// [Effect reference - Oscillator]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#oscillator
        Oscillator        = FMOD_DSP_TYPE_OSCILLATOR,
        /// Filters sound using a high quality, resonant lowpass filter
        /// algorithm but consumes more CPU time. Deprecated and will be removed
        /// in a future release. See [DspLowpass] remarks for parameter
        /// information, [Effect reference - Low Pass] for overview.
        ///
        /// [Effect reference - Low Pass]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#it-low-pass
        #[deprecated(note = "This filter is deprecated and will be removed in a future release.")]
        Lowpass           = FMOD_DSP_TYPE_LOWPASS,
        /// Filters sound using a resonant lowpass filter algorithm that is used
        /// in Impulse Tracker, but with limited cutoff range (0 to 8060hz). See
        /// [DspItLowpass] for parameter information,
        /// [Effect reference - IT Low Pass] for overview.
        ///
        /// [Effect reference - IT Low Pass]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#it-low-pass
        ItLowpass         = FMOD_DSP_TYPE_ITLOWPASS,
        /// Filters sound using a resonant highpass filter algorithm. Deprecated
        /// and will be removed in a future release. See [DspHighpass] remarks
        /// for parameter information, [Effect reference - High Pass] for
        /// overview.
        ///
        /// [Effect reference - High Pass]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#high-pass
        #[deprecated(note = "This filter is deprecated and will be removed in a future release.")]
        Highpass          = FMOD_DSP_TYPE_HIGHPASS,
        /// Produces an echo on the sound and fades out at the desired rate. See
        /// [DspEcho] for parameter information, [Effect reference - Echo] for
        /// overview.
        ///
        /// [Effect reference - Echo]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#echo
        Echo              = FMOD_DSP_TYPE_ECHO,
        /// Pans and scales the volume of a unit. See [DspFader] for parameter
        /// information, [Effect reference - Fader] for overview.
        ///
        /// [Effect reference - Fader]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#fader
        Fader             = FMOD_DSP_TYPE_FADER,
        /// Produces a flange effect on the sound. See [DspFlange] for parameter
        /// information, [Effect reference - Flange] for overview.
        ///
        /// [Effect reference - Flange]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#flange
        Flange            = FMOD_DSP_TYPE_FLANGE,
        /// Distorts the sound. See [DspDistortion] for parameter information,
        /// [Effect reference - Distortion] for overview.
        ///
        /// [Effect reference - Distortion]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#distortion
        Distortion        = FMOD_DSP_TYPE_DISTORTION,
        /// Normalizes or amplifies the sound to a certain level. See
        /// [DspNormalize] for parameter information,
        /// [Effect reference - Normalize] for overview.
        ///
        /// [Effect reference - Normalize]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#normalize
        Normalize         = FMOD_DSP_TYPE_NORMALIZE,
        /// Limits the sound to a certain level. See [DspLimiter] for parameter
        /// information, [Effect reference - Limiter] for overview.
        ///
        /// [Effect reference - Limiter]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#limiter
        Limiter           = FMOD_DSP_TYPE_LIMITER,
        /// Attenuates or amplifies a selected frequency range. Deprecated and
        /// will be removed in a future release. See [Dsp::ParamEq] for
        /// parameter information, [Effect reference - Parametric EQ] for
        /// overview.
        ///
        /// [Effect reference - Parametric EQ]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#parametric-eq
        #[deprecated(note = "This filter is deprecated and will be removed in a future release.")]
        ParamEq           = FMOD_DSP_TYPE_PARAMEQ,
        /// Bends the pitch of a sound without changing the speed of playback.
        /// See [DspPitchShift] for parameter information,
        /// [Effect reference - Pitch Shifter] for overview.
        ///
        /// [Effect reference - Pitch Shifter]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#pitch-shifter
        PitchShift        = FMOD_DSP_TYPE_PITCHSHIFT,
        /// Produces a chorus effect on the sound. See [DspChorus] for parameter
        /// information, [Effect reference - Chorus] for overview.
        ///
        /// [Effect reference - Chorus]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#chorus
        Chorus            = FMOD_DSP_TYPE_CHORUS,
        /// Allows the use of Steinberg VST plugins.
        VstPlugin         = FMOD_DSP_TYPE_VSTPLUGIN,
        /// Allows the use of Nullsoft Winamp plugins.
        WinampPlugin      = FMOD_DSP_TYPE_WINAMPPLUGIN,
        /// Produces an echo on the sound and fades out at the desired rate as
        /// is used in Impulse Tracker. See [DspItEcho] for parameter
        /// information, [Effect reference - IT Echo] for overview.
        ///
        /// [Effect reference - IT Echo]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#it-echo
        ItEcho            = FMOD_DSP_TYPE_ITECHO,
        /// Dynamic compression (linked/unlinked multi-channel, wideband). See
        /// [DspCompressor] for parameter information,
        /// [Effect reference - Compressor] for overview.
        ///
        /// [Effect reference - Compressor]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#compressor
        Compressor        = FMOD_DSP_TYPE_COMPRESSOR,
        /// I3DL2 reverb effect. See [DspSfxReverb] for parameter information,
        /// [Effect reference - SFX Reverb] for overview.
        ///
        /// [Effect reference - SFX Reverb]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#sfx-reverb
        SfxReverb         = FMOD_DSP_TYPE_SFXREVERB,
        /// Filters sound using a simple lowpass with no resonance, but has
        /// flexible cutoff and is fast. Deprecated and will be removed in a
        /// future release. See [DspLowpassSimple] remarks for parameter
        /// information, [Effect reference - Low Pass Simple] for overview.
        ///
        /// [Effect reference - Low Pass Simple]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#low-pass-simple
        #[deprecated(note = "This filter is deprecated and will be removed in a future release.")]
        LowpassSimple     = FMOD_DSP_TYPE_LOWPASS_SIMPLE,
        /// Produces different delays on individual channels of the sound. See
        /// [DspDelay] for parameter information, [Effect reference - Delay]
        /// for overview.
        ///
        /// [Effect reference - Delay]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#delay
        Delay             = FMOD_DSP_TYPE_DELAY,
        /// Produces a tremolo / chopper effect on the sound. See [DspTremelo]
        /// for parameter information, [Effect reference - Tremolo] for
        /// overview.
        ///
        /// [Effect reference - Tremolo]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#tremolo
        Tremolo           = FMOD_DSP_TYPE_TREMOLO,
        /// Unsupported / Deprecated.
        #[deprecated(note = "This filter is unsupported and will be removed in a future release.")]
        LadspaPlugin      = FMOD_DSP_TYPE_LADSPAPLUGIN,
        /// Sends a copy of the signal to a return DSP anywhere in the DSP tree.
        /// See [DspSend] for parameter information, [Effect reference - Send]
        /// for overview.
        ///
        /// [Effect reference - Send]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#send
        Send              = FMOD_DSP_TYPE_SEND,
        /// Receives signals from a number of send DSPs. See [DspReturn] for
        /// parameter information, [Effect reference - Return] for overview.
        ///
        /// [Effect reference - Return]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#return
        Return            = FMOD_DSP_TYPE_RETURN,
        /// Filters sound using a simple highpass with no resonance, but has
        /// flexible cutoff and is fast. Deprecated and will be removed in a
        /// future release. See [DspHighpassSimple] remarks for parameter
        /// information, [Effect reference - High Pass Simple] for overview.
        ///
        /// [Effect reference - High Pass Simple]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#high-pass-simple
        #[deprecated(note = "This filter is deprecated and will be removed in a future release.")]
        HighpassSimple    = FMOD_DSP_TYPE_HIGHPASS_SIMPLE,
        /// Pans the signal in 2D or 3D, possibly upmixing or downmixing as
        /// well. See [DspPan] for parameter information,
        /// [Effect reference - Pan] for overview.
        ///
        /// [Effect reference - Pan]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#pan
        Pan               = FMOD_DSP_TYPE_PAN,
        /// Three-band equalizer. See [DspThreeEq] for parameter information,
        /// [Effect reference - Three EQ] for overview.
        ///
        /// [Effect reference - Three EQ]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#three-eq
        ThreeEq           = FMOD_DSP_TYPE_THREE_EQ,
        /// Analyzes the signal and provides spectrum information back through
        /// get_parameter. See [DspFft] for parameter information,
        /// [Effect reference - FFT] for overview.
        ///
        /// [Effect reference - FFT]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#fft
        Fft               = FMOD_DSP_TYPE_FFT,
        /// Analyzes the loudness and true peak of the signal.
        LoudnessMeter     = FMOD_DSP_TYPE_LOUDNESS_METER,
        /// Tracks the envelope of the input/sidechain signal. Deprecated and
        /// will be removed in a future release. See [DspEnvelopeFollower] for
        /// parameter information, [Effect reference - Envelope Follower] for
        /// overview.
        ///
        /// [Effect reference - Envelope Follower]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#envelope-follower
        EnvelopeFollower  = FMOD_DSP_TYPE_ENVELOPEFOLLOWER,
        /// Convolution reverb. See [DspConvolutionReverb] for parameter
        /// information, [Effect reference - Convolution Reverb] for overview.
        ///
        /// [Effect reference - Convolution Reverb]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#convolution-reverb
        ConvolutionReverb = FMOD_DSP_TYPE_CONVOLUTIONREVERB,
        /// Provides per channel gain, channel grouping of the input signal
        /// which also sets the speaker format for the output signal, and
        /// customizable input to output channel routing. See [DspChannelMix]
        /// for parameter information, [Effect reference - Channel Mix] for
        /// overview.
        ///
        /// [Effect reference - Channel Mix]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#channel-mix
        ChannelMix        = FMOD_DSP_TYPE_CHANNELMIX,
        /// 'sends' and 'receives' from a selection of up to 32 different slots.
        /// It is like a send/return but it uses global slots rather than
        /// returns as the destination. It also has other features. Multiple
        /// transceivers can receive from a single channel, or multiple
        /// transceivers can send to a single channel, or a combination of both.
        /// See [DspTransceiver] for parameter information,
        /// [Effect reference - Transceiver] for overview.
        ///
        /// [Effect reference - Transceiver]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#transceiver
        Transceiver       = FMOD_DSP_TYPE_TRANSCEIVER,
        /// Spatializes input signal by passing it to an external object mixer.
        /// See [DspObjectPan] for parameter information,
        /// [Effect reference - Object Panner] for overview.
        ///
        /// [Effect reference - Object Panner]: https://fmod.com/resources/documentation-api?version=2.02&page=effects-reference.html#object-panner
        ObjectPan         = FMOD_DSP_TYPE_OBJECTPAN,
        /// Five band parametric equalizer. See [DspMultibandEq] for parameter
        /// information, [Effect reference - Multiband Equalizer] for overview.
        MultibandEq       = FMOD_DSP_TYPE_MULTIBAND_EQ,
    }
}

enum_struct! {
    /// Channel Mix DSP parameter types.
    ///
    /// For [DspChannelMix::OutputGrouping], this value will set the output
    /// speaker format for the DSP which determines the number of output
    /// channels.
    ///
    /// For input channels mapped to an output channel in excess of the number
    /// of output channels, it will instead be mapped to the modulo of that
    /// channel index. Eg if there are 4 output channels, the input channel
    /// mapped to output channel index 5 will be mapped to index 1.
    pub enum DspChannelMix: i32 {
        /// Channel mix output grouping.
        OutputGrouping = FMOD_DSP_CHANNELMIX_OUTPUTGROUPING,
        /// Channel #0 gain.
        GainCh0        = FMOD_DSP_CHANNELMIX_GAIN_CH0,
        /// Channel #1 gain.
        GainCh1        = FMOD_DSP_CHANNELMIX_GAIN_CH1,
        /// Channel #2 gain.
        GainCh2        = FMOD_DSP_CHANNELMIX_GAIN_CH2,
        /// Channel #3 gain.
        GainCh3        = FMOD_DSP_CHANNELMIX_GAIN_CH3,
        /// Channel #4 gain.
        GainCh4        = FMOD_DSP_CHANNELMIX_GAIN_CH4,
        /// Channel #5 gain.
        GainCh5        = FMOD_DSP_CHANNELMIX_GAIN_CH5,
        /// Channel #6 gain.
        GainCh6        = FMOD_DSP_CHANNELMIX_GAIN_CH6,
        /// Channel #7 gain.
        GainCh7        = FMOD_DSP_CHANNELMIX_GAIN_CH7,
        /// Channel #8 gain.
        GainCh8        = FMOD_DSP_CHANNELMIX_GAIN_CH8,
        /// Channel #9 gain.
        GainCh9        = FMOD_DSP_CHANNELMIX_GAIN_CH9,
        /// Channel #10 gain.
        GainCh10       = FMOD_DSP_CHANNELMIX_GAIN_CH10,
        /// Channel #11 gain.
        GainCh11       = FMOD_DSP_CHANNELMIX_GAIN_CH11,
        /// Channel #12 gain.
        GainCh12       = FMOD_DSP_CHANNELMIX_GAIN_CH12,
        /// Channel #13 gain.
        GainCh13       = FMOD_DSP_CHANNELMIX_GAIN_CH13,
        /// Channel #14 gain.
        GainCh14       = FMOD_DSP_CHANNELMIX_GAIN_CH14,
        /// Channel #15 gain.
        GainCh15       = FMOD_DSP_CHANNELMIX_GAIN_CH15,
        /// Channel #16 gain.
        GainCh16       = FMOD_DSP_CHANNELMIX_GAIN_CH16,
        /// Channel #17 gain.
        GainCh17       = FMOD_DSP_CHANNELMIX_GAIN_CH17,
        /// Channel #18 gain.
        GainCh18       = FMOD_DSP_CHANNELMIX_GAIN_CH18,
        /// Channel #19 gain.
        GainCh19       = FMOD_DSP_CHANNELMIX_GAIN_CH19,
        /// Channel #20 gain.
        GainCh20       = FMOD_DSP_CHANNELMIX_GAIN_CH20,
        /// Channel #21 gain.
        GainCh21       = FMOD_DSP_CHANNELMIX_GAIN_CH21,
        /// Channel #22 gain.
        GainCh22       = FMOD_DSP_CHANNELMIX_GAIN_CH22,
        /// Channel #23 gain.
        GainCh23       = FMOD_DSP_CHANNELMIX_GAIN_CH23,
        /// Channel #24 gain.
        GainCh24       = FMOD_DSP_CHANNELMIX_GAIN_CH24,
        /// Channel #25 gain.
        GainCh25       = FMOD_DSP_CHANNELMIX_GAIN_CH25,
        /// Channel #26 gain.
        GainCh26       = FMOD_DSP_CHANNELMIX_GAIN_CH26,
        /// Channel #27 gain.
        GainCh27       = FMOD_DSP_CHANNELMIX_GAIN_CH27,
        /// Channel #28 gain.
        GainCh28       = FMOD_DSP_CHANNELMIX_GAIN_CH28,
        /// Channel #29 gain.
        GainCh29       = FMOD_DSP_CHANNELMIX_GAIN_CH29,
        /// Channel #30 gain.
        GainCh30       = FMOD_DSP_CHANNELMIX_GAIN_CH30,
        /// Channel #31 gain.
        GainCh31       = FMOD_DSP_CHANNELMIX_GAIN_CH31,
        /// Output channel for Input channel #0
        OutputCh0      = FMOD_DSP_CHANNELMIX_OUTPUT_CH0,
        /// Output channel for Input channel #1
        OutputCh1      = FMOD_DSP_CHANNELMIX_OUTPUT_CH1,
        /// Output channel for Input channel #2
        OutputCh2      = FMOD_DSP_CHANNELMIX_OUTPUT_CH2,
        /// Output channel for Input channel #3
        OutputCh3      = FMOD_DSP_CHANNELMIX_OUTPUT_CH3,
        /// Output channel for Input channel #4
        OutputCh4      = FMOD_DSP_CHANNELMIX_OUTPUT_CH4,
        /// Output channel for Input channel #5
        OutputCh5      = FMOD_DSP_CHANNELMIX_OUTPUT_CH5,
        /// Output channel for Input channel #6
        OutputCh6      = FMOD_DSP_CHANNELMIX_OUTPUT_CH6,
        /// Output channel for Input channel #7
        OutputCh7      = FMOD_DSP_CHANNELMIX_OUTPUT_CH7,
        /// Output channel for Input channel #8
        OutputCh8      = FMOD_DSP_CHANNELMIX_OUTPUT_CH8,
        /// Output channel for Input channel #9
        OutputCh9      = FMOD_DSP_CHANNELMIX_OUTPUT_CH9,
        /// Output channel for Input channel #10
        OutputCh10     = FMOD_DSP_CHANNELMIX_OUTPUT_CH10,
        /// Output channel for Input channel #11
        OutputCh11     = FMOD_DSP_CHANNELMIX_OUTPUT_CH11,
        /// Output channel for Input channel #12
        OutputCh12     = FMOD_DSP_CHANNELMIX_OUTPUT_CH12,
        /// Output channel for Input channel #13
        OutputCh13     = FMOD_DSP_CHANNELMIX_OUTPUT_CH13,
        /// Output channel for Input channel #14
        OutputCh14     = FMOD_DSP_CHANNELMIX_OUTPUT_CH14,
        /// Output channel for Input channel #15
        OutputCh15     = FMOD_DSP_CHANNELMIX_OUTPUT_CH15,
        /// Output channel for Input channel #16
        OutputCh16     = FMOD_DSP_CHANNELMIX_OUTPUT_CH16,
        /// Output channel for Input channel #17
        OutputCh17     = FMOD_DSP_CHANNELMIX_OUTPUT_CH17,
        /// Output channel for Input channel #18
        OutputCh18     = FMOD_DSP_CHANNELMIX_OUTPUT_CH18,
        /// Output channel for Input channel #19
        OutputCh19     = FMOD_DSP_CHANNELMIX_OUTPUT_CH19,
        /// Output channel for Input channel #20
        OutputCh20     = FMOD_DSP_CHANNELMIX_OUTPUT_CH20,
        /// Output channel for Input channel #21
        OutputCh21     = FMOD_DSP_CHANNELMIX_OUTPUT_CH21,
        /// Output channel for Input channel #22
        OutputCh22     = FMOD_DSP_CHANNELMIX_OUTPUT_CH22,
        /// Output channel for Input channel #23
        OutputCh23     = FMOD_DSP_CHANNELMIX_OUTPUT_CH23,
        /// Output channel for Input channel #24
        OutputCh24     = FMOD_DSP_CHANNELMIX_OUTPUT_CH24,
        /// Output channel for Input channel #25
        OutputCh25     = FMOD_DSP_CHANNELMIX_OUTPUT_CH25,
        /// Output channel for Input channel #26
        OutputCh26     = FMOD_DSP_CHANNELMIX_OUTPUT_CH26,
        /// Output channel for Input channel #27
        OutputCh27     = FMOD_DSP_CHANNELMIX_OUTPUT_CH27,
        /// Output channel for Input channel #28
        OutputCh28     = FMOD_DSP_CHANNELMIX_OUTPUT_CH28,
        /// Output channel for Input channel #29
        OutputCh29     = FMOD_DSP_CHANNELMIX_OUTPUT_CH29,
        /// Output channel for Input channel #30
        OutputCh30     = FMOD_DSP_CHANNELMIX_OUTPUT_CH30,
        /// Output channel for Input channel #31
        OutputCh31     = FMOD_DSP_CHANNELMIX_OUTPUT_CH31,
    }
}
