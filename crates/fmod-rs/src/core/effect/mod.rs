//! These are the parameters for controlling Digital Signal Processors.
//!
//! Effects are categorized in each description for performance as:
//!
//! - **low overhead** - Can be used freely without much CPU impact.
//! - **medium overhead** - Use carefully with a medium CPU impact.
//! - **high overhead** - Use sparingly due to high CPU cost.
//!
//! Always use the FMOD Studio Profiler to determine the CPU impact of effects
//! in your project.

// TODO: docgen from effects-reference.html

#![allow(non_snake_case)]

use {
    fmod::{raw::*, *},
    paste::paste,
    seq_macro::seq,
    std::{mem::size_of, ptr},
};

/// A parameter index for a DSP effect.
pub trait DspParam<T: DspParamType>: Into<i32> {
    /// The type of DSP this parameter is for. Mostly informational.
    const KIND: DspType;
}

impl<T: DspParamType> DspParam<T> for i32 {
    const KIND: DspType = DspType::Unknown;
}

/// A type usable for DSP parameters.
pub trait DspParamType: Sized {
    /// Sets a DSP parameter by index.
    fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &Self) -> Result;
    /// Retrieves a DSP parameter by index.
    fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<Self>;
    /// Retrieves the string representation of a DSP parameter by index.
    fn get_dsp_parameter_string<'a>(dsp: &Dsp, index: i32, bytes: &'a mut [u8]) -> Result<&'a str>;
}

impl DspParamType for bool {
    fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &bool) -> Result {
        ffi!(FMOD_DSP_SetParameterBool(
            dsp.as_raw(),
            index,
            *value as FMOD_BOOL,
        ))?;
        Ok(())
    }

    fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<bool> {
        let mut value = FMOD_BOOL::default();
        ffi!(FMOD_DSP_GetParameterBool(
            dsp.as_raw(),
            index,
            &mut value,
            ptr::null_mut(),
            0,
        ))?;
        Ok(value != 0)
    }

    fn get_dsp_parameter_string<'a>(dsp: &Dsp, index: i32, bytes: &'a mut [u8]) -> Result<&'a str> {
        ffi!(FMOD_DSP_GetParameterBool(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(CStr8::from_utf8_until_nul(bytes).map_err(|_| Error::InvalidString)?)
    }
}

impl<const N: usize> DspParamType for [u8; N] {
    fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &[u8; N]) -> Result {
        ffi!(FMOD_DSP_SetParameterData(
            dsp.as_raw(),
            index,
            value.as_ptr().cast_mut().cast(),
            value.len() as u32,
        ))?;
        Ok(())
    }

    fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<[u8; N]> {
        let mut value = ptr::null_mut();
        let mut length = 0;
        ffi!(FMOD_DSP_GetParameterData(
            dsp.as_raw(),
            index,
            &mut value,
            &mut length,
            ptr::null_mut(),
            0,
        ))?;

        if length as usize >= N {
            yeet!(Error::InvalidParam);
        }

        // FIXME: copying out could possibly race a set, what do?
        Ok(unsafe { *value.cast() })
    }

    fn get_dsp_parameter_string<'a>(dsp: &Dsp, index: i32, bytes: &'a mut [u8]) -> Result<&'a str> {
        ffi!(FMOD_DSP_GetParameterData(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(CStr8::from_utf8_until_nul(bytes).map_err(|_| Error::InvalidString)?)
    }
}

impl DspParamType for f32 {
    fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &f32) -> Result {
        ffi!(FMOD_DSP_SetParameterFloat(dsp.as_raw(), index, *value))?;
        Ok(())
    }

    fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<f32> {
        let mut value = f32::default();
        ffi!(FMOD_DSP_GetParameterFloat(
            dsp.as_raw(),
            index,
            &mut value,
            ptr::null_mut(),
            0,
        ))?;
        Ok(value)
    }

    fn get_dsp_parameter_string<'a>(dsp: &Dsp, index: i32, bytes: &'a mut [u8]) -> Result<&'a str> {
        ffi!(FMOD_DSP_GetParameterFloat(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(CStr8::from_utf8_until_nul(bytes).map_err(|_| Error::InvalidString)?)
    }
}

impl DspParamType for i32 {
    fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &i32) -> Result {
        ffi!(FMOD_DSP_SetParameterInt(dsp.as_raw(), index, *value))?;
        Ok(())
    }

    fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<i32> {
        let mut value = i32::default();
        ffi!(FMOD_DSP_GetParameterInt(
            dsp.as_raw(),
            index,
            &mut value,
            ptr::null_mut(),
            0,
        ))?;
        Ok(value)
    }

    fn get_dsp_parameter_string<'a>(dsp: &Dsp, index: i32, bytes: &'a mut [u8]) -> Result<&'a str> {
        ffi!(FMOD_DSP_GetParameterInt(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(CStr8::from_utf8_until_nul(bytes).map_err(|_| Error::InvalidString)?)
    }
}

macro_rules! dsp_params {
    {
        $Kind:ident =>
        $(
            $(#[$meta:meta])*
            pub struct $Param:ident($Raw:ident): $Type:ty;
        )*
    } => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Param;
        impl DspParam<$Type> for $Param {
            const KIND: DspType = DspType::$Kind;
        }
        impl From<$Param> for i32 {
            #[inline]
            fn from(_: $Param) -> Self {
                $Raw
            }
        }
    )*};
}

/// Channel Mix DSP parameter types.
///
/// For [ChannelMix::OutputGrouping], this value will set the output speaker
/// format for the DSP which determines the number of output channels.
///
/// For input channels mapped to an output channel in excess of the number of
/// output channels, it will instead be mapped to the modulo of that channel
/// index. Eg if there are 4 output channels, the input channel mapped to
/// output channel index 5 will be mapped to index 1.
pub mod ChannelMix {
    use super::*;

    dsp_params! { ChannelMix =>
        /// Channel mix output grouping.
        pub struct OutputGrouping(FMOD_DSP_CHANNELMIX_OUTPUTGROUPING): Output;
    }

    /// Channel #N gain.
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct GainCh<const N: i32>;
    seq! {N in 0..32 {
        impl DspParam<f32> for GainCh<N> {
            const KIND: DspType = DspType::ChannelMix;
        }
        impl From<GainCh<N>> for i32 {
            #[inline]
            #[allow(clippy::identity_op)]
            fn from(_: GainCh<N>) -> Self {
                FMOD_DSP_CHANNELMIX_GAIN_CH0 + N
            }
        }
    }}

    /// Output channel for Input channel #N.
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct OutputCh<const N: i32>;
    seq! {N in 0..32 {
        impl DspParam<i32> for OutputCh<N> {
            const KIND: DspType = DspType::ChannelMix;
        }
        impl From<OutputCh<N>> for i32 {
            #[inline]
            #[allow(clippy::identity_op)]
            fn from(_: OutputCh<N>) -> Self {
                FMOD_DSP_CHANNELMIX_OUTPUT_CH0 + N
            }
        }
    }}

    fmod_enum! {
        #[derive(Default)]
        /// Channel Mix DSP outgrouping parameter types.
        pub enum Output: FMOD_DSP_CHANNELMIX_OUTPUT
        where const { self <= FMOD_DSP_CHANNELMIX_OUTPUT_ALL7POINT1POINT4 }
        {
            #[default]
            /// Output channel count = input channel count. Mapping: See [`Speaker`].
            Default = FMOD_DSP_CHANNELMIX_OUTPUT_DEFAULT,
            /// Output channel count = 1. Mapping: Mono, Mono, Mono, Mono, Mono, Mono, ... (each channel all the way up to [`MAX_CHANNEL_WIDTH`] channels are treated as if they were mono)
            AllMono = FMOD_DSP_CHANNELMIX_OUTPUT_ALLMONO,
            /// Output channel count = 2. Mapping: Left, Right, Left, Right, Left, Right, ... (each pair of channels is treated as stereo all the way up to [`MAX_CHANNEL_WIDTH`] channels)
            AllStereo = FMOD_DSP_CHANNELMIX_OUTPUT_ALLSTEREO,
            /// Output channel count = 4. Mapping: Repeating pattern of Front Left, Front Right, Surround Left, Surround Right.
            AllQuad = FMOD_DSP_CHANNELMIX_OUTPUT_ALLQUAD,
            /// Output channel count = 6. Mapping: Repeating pattern of Front Left, Front Right, Center, LFE, Surround Left, Surround Right.
            AllSurround51 = FMOD_DSP_CHANNELMIX_OUTPUT_ALL5POINT1,
            /// Output channel count = 8. Mapping: Repeating pattern of Front Left, Front Right, Center, LFE, Surround Left, Surround Right, Back Left, Back Right.
            AllSurround71 = FMOD_DSP_CHANNELMIX_OUTPUT_ALL7POINT1,
            /// Output channel count = 6. Mapping: Repeating pattern of LFE in a 5.1 output signal.
            AllLfe = FMOD_DSP_CHANNELMIX_OUTPUT_ALLLFE,
            /// Output channel count = 12. Mapping: Repeating pattern of Front Left, Front Right, Center, LFE, Surround Left, Surround Right, Back Left, Back Right, Top Front Left, Top Front Right, Top Back Left, Top Back Right.
            AllSurround714 = FMOD_DSP_CHANNELMIX_OUTPUT_ALL7POINT1POINT4,
        }
    }
}

/// Chorus DSP parameter types.
///
/// Chorus is an effect where the sound is more 'spacious' due a copy of the
/// signal being played along side the original, but with the delay of each
/// copy modulating on a sine wave. As there are 2 versions of the same signal
/// (dry vs wet), by default each signal is given 50% mix, so that the total
/// is not louder than the original unaffected signal.
pub mod Chorus {
    use super::*;

    dsp_params! { Chorus =>
        /// Percentage of wet signal in mix.
        pub struct Mix(FMOD_DSP_CHORUS_MIX): f32;
        /// Chorus modulation rate.
        pub struct Rate(FMOD_DSP_CHORUS_RATE): f32;
        /// Chorus modulation depth.
        pub struct Depth(FMOD_DSP_CHORUS_DEPTH): f32;
    }
}

/// Compressor DSP parameter types.
///
/// This is a multi-channel software limiter that is uniform across the whole spectrum.  
/// The limiter is not guaranteed to catch every peak above the threshold level,
/// because it cannot apply gain reduction instantaneously - the time delay is
/// determined by the attack time. However setting the attack time too short
/// will distort the sound, so it is a compromise. High level peaks can be
/// avoided by using a short attack time - but not too short, and setting the
/// threshold a few decibels below the critical level.
pub mod Compressor {
    use super::*;

    dsp_params! { Compressor =>
        /// Threshold level.
        pub struct Threshold(FMOD_DSP_COMPRESSOR_THRESHOLD): f32;
        /// Compression Ratio.
        pub struct Ratio(FMOD_DSP_COMPRESSOR_RATIO): f32;
        /// Attack time.
        pub struct Attack(FMOD_DSP_COMPRESSOR_ATTACK): f32;
        /// Release time.
        pub struct Release(FMOD_DSP_COMPRESSOR_RELEASE): f32;
        /// Make-up gain applied after limiting.
        pub struct GainMakeup(FMOD_DSP_COMPRESSOR_GAINMAKEUP): f32;
        /// Whether to analyse the sidechain signal instead of the input signal.
        pub struct UseSidechain(FMOD_DSP_COMPRESSOR_USESIDECHAIN): Sidechain;
        /// An unlinked compression uses a separate compressor per channel.
        pub struct Linked(FMOD_DSP_COMPRESSOR_LINKED): bool;
    }
}

/// Convolution reverb DSP parameter types.
///
/// Convolution reverb is a reverberation effect that uses a recording of a
/// physical space known as an Impulse Response file (or IR file) to generate
/// frequency specific reverberation.
pub mod ConvolutionReverb {
    use super::*;

    dsp_params! { ConvolutionReverb =>
        /// Volume of echo signal to pass to output.
        pub struct Wet(FMOD_DSP_CONVOLUTION_REVERB_PARAM_WET): f32;
        /// Original sound volume.
        pub struct Dry(FMOD_DSP_CONVOLUTION_REVERB_PARAM_DRY): f32;
        /// Linked - channels are mixed together before processing through the reverb.
        pub struct Linked(FMOD_DSP_CONVOLUTION_REVERB_PARAM_LINKED): bool;
    }

    // ImpulseResponse
}

/// Delay DSP parameter types.
///
/// Note. Every time MaxDelay is changed, the plugin re-allocates the delay
/// buffer. This means the delay will disappear at that time while it refills
/// its new buffer. A larger MaxDelay results in larger amounts of memory
/// allocated.
///
/// Channel delays above MaxDelay will be clipped to MaxDelay and the delay
/// buffer will not be resized.
pub mod Delay {
    use super::*;

    dsp_params! { Delay =>
        /// Maximum delay, for memory allocation purposes.
        pub struct MaxDelay(FMOD_DSP_DELAY_MAXDELAY): f32;
    }

    /// Channel #N Delay.
    pub struct DelayCh<const N: i32>;
    seq! {N in 0..16 {
        impl DspParam<f32> for DelayCh<N> {
            const KIND: DspType = DspType::Delay;
        }
        impl From<DelayCh<N>> for i32 {
            #[inline]
            #[allow(clippy::identity_op)]
            fn from(_: DelayCh<N>) -> Self {
                FMOD_DSP_DELAY_CH0 + N
            }
        }
    }}
}

/// Distortion DSP parameter types.
pub mod Distortion {
    pub use super::*;

    dsp_params! { Distortion =>
        /// Distortion value.
        pub struct Level(FMOD_DSP_DISTORTION_LEVEL): f32;
    }
}

/// Echo DSP parameter types.
///
/// Note. Every time the delay is changed, the plugin re-allocates the echo
/// buffer. This means the echo will disappear at that time while it refills its
/// new buffer. Larger echo delays result in larger amounts of memory allocated.
pub mod Echo {
    use super::*;

    dsp_params! { Echo =>
        /// Echo delay.
        pub struct Delay(FMOD_DSP_ECHO_DELAY): f32;
        /// Echo decay per delay. 100.0 = No decay, 0.0 = total decay.
        pub struct Feedback(FMOD_DSP_ECHO_FEEDBACK): f32;
        /// Original sound volume.
        pub struct DryLevel(FMOD_DSP_ECHO_DRYLEVEL): f32;
        /// Volume of echo signal to pass to output.
        pub struct WetLevel(FMOD_DSP_ECHO_WETLEVEL): f32;
    }
}

/// Parameter types for the [`DspType::EnvelopeFollower`] unit. This is a simple
/// envelope follower for tracking the signal level. This unit does not affect
/// the incoming signal.
#[deprecated = "Deprecated and will be removed in a future release."]
pub mod EnvelopeFollower {
    use super::*;

    dsp_params! { EnvelopeFollower =>
        /// Attack time.
        pub struct Attack(FMOD_DSP_ENVELOPEFOLLOWER_ATTACK): f32;
        /// Release time.
        pub struct Release(FMOD_DSP_ENVELOPEFOLLOWER_RELEASE): f32;
        /// Current value of the envelope.
        pub struct Envelope(FMOD_DSP_ENVELOPEFOLLOWER_ENVELOPE): f32;
        /// Whether to analyse the sidechain signal instead of the input signal.
        pub struct UseSidechain(FMOD_DSP_ENVELOPEFOLLOWER_USESIDECHAIN): Sidechain;
    }
}

/// Fader DSP parameter types.
pub mod Fader {
    use super::*;

    dsp_params! { Fader =>
        /// Signal gain.
        pub struct Gain(FMOD_DSP_FADER_GAIN): f32;
        /// Overall gain to allow FMOD to know the DSP is scaling the signal
        /// for visualization purposes.
        pub struct OverallGain(FMOD_DSP_FADER_OVERALL_GAIN): super::OverallGain;
    }
}

/// FFT DSP parameter types.
///
/// Set the attributes for the spectrum analysis with [`Fft::WindowSize`] and
/// [`Fft::WindowType`], and retrieve the results with [`Fft::SpectrumData`] and
/// [`Fft::DominantFreq`].
///
/// [`Fft::SpectrumData`] stores its data in the [`Fft::DataType`]. You will
/// need to cast to this structure to get the right data.
pub mod Fft {
    use super::*;

    dsp_params! { Fft =>
        /// Window size. Must be a power of 2 between 128 and 16384.
        pub struct WindowSize(FMOD_DSP_FFT_WINDOWSIZE): i32;
        /// FFT Window Type.
        pub struct WindowType(FMOD_DSP_FFT_WINDOWTYPE): Window;
        /// Returns the dominant frequencies for each channel.
        pub struct DominantFreq(FMOD_DSP_FFT_DOMINANT_FREQ): f32;
    }

    // SpectrumData

    fmod_enum! {
        #[derive(Default)]
        /// List of windowing methods for the FFT DSP.
        ///
        /// Used in spectrum analysis to reduce leakage / transient signals
        /// interfering with the analysis. This is a problem with analysis of
        /// continuous signals that only have a small portion of the signal
        /// sample (the fft window size). Windowing the signal with a curve or
        /// triangle tapers the sides of the fft window to help alleviate this
        /// problem.
        ///
        /// Cyclic signals such as a sine wave that repeat their cycle in a
        /// multiple of the window size do not need windowing. I.e. If the sine
        /// wave repeats every 1024, 512, 256 etc samples and the FMOD fft
        /// window is 1024, then the signal would not need windowing.
        ///
        /// Not windowing is the same as [`Window::Rect`], which is the default.
        //  TODO: ???? FMOD_DSP_FFT_WINDOWTYPE says FMOD_DSP_FFT_WINDOW_HAMMING
        /// If the cycle of the signal (ie the sine wave) is not a multiple of
        /// the window size, it will cause frequency abnormalities, so a
        /// different windowing method is needed.
        pub enum Window: FMOD_DSP_FFT_WINDOW
        where const { self <= FMOD_DSP_FFT_WINDOW_BLACKMANHARRIS }
        {
            #[default]
            /// w[n] = 1.0
            Rect = FMOD_DSP_FFT_WINDOW_RECT,
            /// w[n] = TRI(2n/N)
            Triangle = FMOD_DSP_FFT_WINDOW_TRIANGLE,
            /// w[n] = 0.54 - (0.46 * COS(n/N) )
            Hamming = FMOD_DSP_FFT_WINDOW_HAMMING,
            /// w[n] = 0.5 * (1.0 - COS(n/N) )
            Hanning = FMOD_DSP_FFT_WINDOW_HANNING,
            /// w[n] = 0.42 - (0.5 * COS(n/N) ) + (0.08 * COS(2.0 * n/N) )
            Blackman = FMOD_DSP_FFT_WINDOW_BLACKMAN,
            /// w[n] = 0.35875 - (0.48829 * COS(1.0 * n/N)) + (0.14128 * COS(2.0 * n/N)) - (0.01168 * COS(3.0 * n/N))
            BlackManHarris = FMOD_DSP_FFT_WINDOW_BLACKMANHARRIS,
        }
    }
}

/// Flange DSP parameter types.
///
/// Flange is an effect where the signal is played twice at the same time, and
/// one copy slides back and forth creating a whooshing or flanging effect. As
/// there are 2 versions of the same signal (dry vs wet), by default each signal
/// is given 50% mix, so that the total is not louder than the original
/// unaffected signal.
///
/// Flange depth is a percentage of a 10ms shift from the original signal.
/// Anything above 10ms is not considered flange because to the ear it begins
/// to 'echo' so 10ms is the highest value possible.
pub mod Flange {
    use super::*;

    dsp_params! { Flange =>
        /// Percentage of wet signal in mix.
        pub struct Mix(FMOD_DSP_FLANGE_MIX): f32;
        /// Flange depth.
        pub struct Depth(FMOD_DSP_FLANGE_DEPTH): f32;
        /// Flange speed.
        pub struct Rate(FMOD_DSP_FLANGE_RATE): f32;
    }
}

/// Highpass DSP parameter types.
///
/// Deprecated and will be removed in a future release, to emulate with
/// [`DspType::MultibandEq`]:
///
/// ```rust,no_run
/// # let system = fmod::System::new()?;
/// # let multiband = system.create_dsp_by_type(fmod::DspType::MultibandEq)?;
/// # let frequency = 5000.0;
/// # let resonance = 1.0;
/// // Configure a single band (band A) as a highpass (all other bands default to off).
/// // 12dB roll-off to approximate the old effect curve.
/// // Cutoff frequency can be used the same as with the old effect.
/// // Resonance can be applied by setting the 'Q' value of the new effect.
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Filter, fmod::effect::MultibandEq::Filter::Highpass12Db)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Cutoff, frequency)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Resonance, resonance)?;
/// # Ok::<_, fmod::Error>(())
/// ```
#[deprecated = "Deprecated and will be removed in a future release."]
#[allow(deprecated)]
pub mod Highpass {
    use super::*;

    dsp_params! { Highpass =>
        /// Highpass cutoff frequency.
        pub struct Cutoff(FMOD_DSP_HIGHPASS_CUTOFF): f32;
        /// Highpass resonance Q value.
        pub struct Resonance(FMOD_DSP_HIGHPASS_RESONANCE): f32;
    }
}

/// Simple Highpass DSP parameter types.
///
/// This is a very simple single-order high pass filter. The emphasis is on
/// speed rather than accuracy, so this should not be used for task requiring
/// critical filtering.
///
/// Deprecated and will be removed in a future release, to emulate with
/// [`DspType::MultibandEq`]:
///
/// ```rust,no_run
/// # let system = fmod::System::new()?;
/// # let multiband = system.create_dsp_by_type(fmod::DspType::MultibandEq)?;
/// # let frequency = 5000.0;
/// // Configure a single band (band A) as a highpass (all other bands default to off).
/// // 12dB roll-off to approximate the old effect curve.
/// // Cutoff frequency can be used the same as with the old effect.
/// // Resonance / 'Q' should remain at default 0.707.
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Filter, fmod::effect::MultibandEq::Filter::Highpass12Db)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Cutoff, frequency)?;
/// # Ok::<_, fmod::Error>(())
/// ```
#[deprecated = "Deprecated and will be removed in a future release."]
#[allow(deprecated)]
pub mod HighpassSimple {
    use super::*;

    dsp_params! { HighpassSimple =>
        /// Highpass cutoff frequency.
        pub struct Cutoff(FMOD_DSP_HIGHPASS_SIMPLE_CUTOFF): f32;
    }
}

/// IT Echo DSP parameter types.
///
/// This is effectively a software based echo filter that emulates the DirectX
/// DMO echo effect. Impulse tracker files can support this, and FMOD will
/// produce the effect on ANY platform, not just those that support DirectX
/// effects!
///
/// Note. Every time the delay is changed, the plugin re-allocates the echo
/// buffer. This means the echo will dissapear at that time while it refills its
/// new buffer. Larger echo delays result in larger amounts of memory allocated.
///
/// As this is a stereo filter made mainly for IT playback, it is targeted for
/// stereo signals. With mono signals only the [`ItEcho::LeftDelay`] is used.
/// For multi-channel signals (>2) there will be no echo on those channels.
pub mod ItEcho {
    use super::*;

    dsp_params! { ItEcho =>
        /// Ratio of wet (processed) signal to dry (unprocessed) signal. Higher is wetter.
        pub struct WetDryMix(FMOD_DSP_ITECHO_WETDRYMIX): f32;
        /// Percentage of output fed back into input.
        pub struct Feedback(FMOD_DSP_ITECHO_FEEDBACK): f32;
        /// Delay for left channel.
        pub struct LeftDelay(FMOD_DSP_ITECHO_LEFTDELAY): f32;
        /// Delay for right channel.
        pub struct RightDelay(FMOD_DSP_ITECHO_RIGHTDELAY): f32;
        /// Value that specifies whether to swap left and right delays with each successive echo. CURRENTLY NOT SUPPORTED.
        pub struct PanDelay(FMOD_DSP_ITECHO_PANDELAY): f32;
    }
}

/// Lowpass DSP parameter types.
///
/// FMOD Studio's .IT playback uses this filter.
///
/// This is different to the default [`DspType::ItLowpass`] filter in that it
/// uses a different quality algorithm and is the filter used to produce the
/// correct sounding playback in .IT files.
///
/// Note! This filter actually has a limited cutoff frequency below the
/// specified maximum, due to its limited design, so for a more open range
/// filter use [`Lowpass`] or if you don't mind not having resonance,
/// [`LowpassSimple`].
///
/// The effective maximum cutoff is about 8060hz.
pub mod ItLowpass {
    use super::*;

    dsp_params! { ItLowpass =>
        /// Lowpass cutoff frequency.
        pub struct Cutoff(FMOD_DSP_ITLOWPASS_CUTOFF): f32;
        /// Lowpass resonance Q value.
        pub struct Resonance(FMOD_DSP_ITLOWPASS_RESONANCE): f32;
    }
}

/// Limited DSP parameter types.
pub mod Limiter {
    use super::*;

    dsp_params! { Limiter =>
        /// Time to return the gain reduction to full in ms.
        pub struct ReleaseTime(FMOD_DSP_LIMITER_RELEASETIME): f32;
        /// Maximum level of the output signal.
        pub struct Ceiling(FMOD_DSP_LIMITER_CEILING): f32;
        /// Maximum amplification allowed.
        pub struct MaximizerGain(FMOD_DSP_LIMITER_MAXIMIZERGAIN): f32;
        /// Channel processing mode where false is independent (limiter per channel)
        /// and true is linked (all channels are summed together before processing).
        pub struct LimiterMode(FMOD_DSP_LIMITER_MODE): bool;
    }
}

/// Loudness meter DSP parameter types.
pub mod LoudnessMeter {
    use super::*;

    dsp_params! { LoudnessMeter =>
        /// Update state.
        pub struct State(FMOD_DSP_LOUDNESS_METER_STATE): StateType;
        /// Channel weighting.
        pub struct Weighting(FMOD_DSP_LOUDNESS_METER_WEIGHTING): WeightingType;
        /// Metering information.
        pub struct MeterInfo(FMOD_DSP_LOUDNESS_METER_INFO): MeterInfoType;
    }

    fmod_enum! {
        /// Loudness meter state indicating update behavior.
        pub enum StateType: FMOD_DSP_LOUDNESS_METER_STATE_TYPE
        where
            const { self <= FMOD_DSP_LOUDNESS_METER_STATE_ANALYZING },
            const { self >= FMOD_DSP_LOUDNESS_METER_STATE_RESET_INTEGRATED },
        {
            /// Reset loudness meter information except max peak.
            ResetIntegrated = FMOD_DSP_LOUDNESS_METER_STATE_RESET_INTEGRATED,
            /// Reset loudness meter max peak.
            ResetMaxPeak = FMOD_DSP_LOUDNESS_METER_STATE_RESET_MAXPEAK,
            /// Reset all loudness meter information.
            ResetAll = FMOD_DSP_LOUDNESS_METER_STATE_RESET_ALL,
            /// Pause loudness meter.
            Paused = FMOD_DSP_LOUDNESS_METER_STATE_PAUSED,
            /// Enable loudness meter recording and analyzing.
            Analyzing = FMOD_DSP_LOUDNESS_METER_STATE_ANALYZING,
        }
    }

    fmod_struct! {
        /// Loudness meter channel weighting.
        pub struct WeightingType = FMOD_DSP_LOUDNESS_METER_WEIGHTING_TYPE {
            /// The weighting of each channel used in calculating loudness.
            #[default([
                1.0, 1.0, 1.0, 0.0, 1.4, 1.4, 1.4, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ])]
            pub channel_weight: [f32; 32],
        }
    }

    fmod_struct! {
        /// Loudness meter information data structure.
        pub struct MeterInfoType = FMOD_DSP_LOUDNESS_METER_INFO_TYPE {
            /// Loudness value indicating current loudness. Calculated using a 400ms window.
            pub momentary_loudness: f32,
            /// Loudness value indicating loudness averaged over a short time duration. Calculated using a 3 second window.
            pub short_term_loudness: f32,
            /// Loudness value indicating loudness over the entire duration of the recording period.
            pub integrated_loudness: f32,
            /// 10th percentile loudness (towards lowest loudness). Uses short term loudness values (3 second averages).
            pub loudness_10th_percentile: f32,
            /// 95th percentile loudness (towards highest loudness). Uses short term loudness values (3 second averages).
            pub loundness_95th_percentile: f32,
            /// Array containing distribution of loudness values. Each array entry is a count of the momentary loudness values
            /// (400ms averages) evenly distributed along the range [-60, 6] excluding loudness values outside that range.
            #[default([0.0; 66])]
            pub loundness_histogram: [f32; 66],
            /// Highest peak.
            pub max_true_peak: f32,
            /// Highest momentary loudness value (400ms averages).
            pub max_momentary_loundness: f32,
        }
    }
}

/// Lowpass DSP parameter types.
///
/// Deprecated and will be removed in a future release, to emulate with
/// [`DspType::MultibandEq`]:
///
/// ```rust,no_run
/// # let system = fmod::System::new()?;
/// # let multiband = system.create_dsp_by_type(fmod::DspType::MultibandEq)?;
/// # let frequency = 5000.0;
/// # let resonance = 1.0;
/// // Configure a single band (band A) as a highpass (all other bands default to off).
/// // 24dB roll-off to approximate the old effect curve.
/// // Cutoff frequency can be used the same as with the old effect.
/// // Resonance can be applied by setting the 'Q' value of the new effect.
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Filter, fmod::effect::MultibandEq::Filter::Lowpass24Db)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Cutoff, frequency)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Resonance, resonance)?;
/// # Ok::<_, fmod::Error>(())
/// ```
#[deprecated = "Deprecated and will be removed in a future release."]
#[allow(deprecated)]
pub mod Lowpass {
    use super::*;

    dsp_params! { Lowpass =>
        /// Lowpass cutoff frequency.
        pub struct Cutoff(FMOD_DSP_LOWPASS_CUTOFF): f32;
        /// Lowpass resonance Q value.
        pub struct Resonance(FMOD_DSP_LOWPASS_RESONANCE): f32;
    }
}

/// Simple Lowpass DSP Parameter types.
///
/// This is a very simple low pass filter, based on two single-pole RC time-constant modules.
///
/// The emphasis is on speed rather than accuracy, so this should not be used for task requiring critical filtering.
///
/// Deprecated and will be removed in a future release, to emulate with
/// [`DspType::MultibandEq`]:
///
/// ```rust,no_run
/// # let system = fmod::System::new()?;
/// # let multiband = system.create_dsp_by_type(fmod::DspType::MultibandEq)?;
/// # let frequency = 5000.0;
/// # let resonance = 1.0;
/// // Configure a single band (band A) as a highpass (all other bands default to off).
/// // 24dB roll-off to approximate the old effect curve.
/// // Cutoff frequency can be used the same as with the old effect.
/// // Resonance / 'Q' should remain at default 0.707.
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Filter, fmod::effect::MultibandEq::Filter::Lowpass24Db)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Cutoff, frequency)?;
/// # Ok::<_, fmod::Error>(())
/// ```
#[deprecated = "Deprecated and will be removed in a future release."]
#[allow(deprecated)]
pub mod LowpassSimple {
    use super::*;

    dsp_params! { LowpassSimple =>
        /// Lowpass cutoff frequency.
        pub struct Cutoff(FMOD_DSP_LOWPASS_SIMPLE_CUTOFF): f32;
    }
}

/// Multiband EQ DSP parameter types.
///
/// Flexible five band parametric equalizer.
pub mod MultibandEq {
    use super::*;

    macro_rules! band {
        ($($A:ident),* $(,)?) => {$( paste! {
            #[doc = "Band " $A]
            pub mod $A {
                use super::*;

                dsp_params! { MultibandEq =>
                    /// Used to interpret the behavior of the remaining parameters.
                    pub struct Filter([<FMOD_DSP_MULTIBAND_EQ_ $A _FILTER>]): FilterType;
                    /// Significant frequency, cutoff [low/high pass, low/high shelf],
                    /// center [notch, peaking, band-pass], phase transition point [all-pass].
                    pub struct Frequency([<FMOD_DSP_MULTIBAND_EQ_ $A _FREQUENCY>]): f32;
                    /// Quality factor, resonance [low/high pass], bandwidth [notch, peaking, band-pass],
                    /// phase transition sharpness [all-pass], unused [low/high shelf].
                    pub struct Q([<FMOD_DSP_MULTIBAND_EQ_ $A _Q>]): f32;
                    /// Boost or attenuation in dB [peaking, high/low shelf only]. -30 to 30. Default = 0.
                    pub struct Gain([<FMOD_DSP_MULTIBAND_EQ_ $A _GAIN>]): f32;
                }
            }
        })*};
    }

    band!(A, B, C, D, E);

    fmod_enum! {
        #[derive(Default)]
        /// Multiband EQ Filter types.
        pub enum FilterType: FMOD_DSP_MULTIBAND_EQ_FILTER_TYPE
        where const { self <= FMOD_DSP_MULTIBAND_EQ_FILTER_ALLPASS }
        {
            #[default]
            /// Disabled filter, no processing.
            Disabled = FMOD_DSP_MULTIBAND_EQ_FILTER_DISABLED,
            /// Resonant low-pass filter, attenuates frequencies (12dB per octave) above
            /// a given point (with specificed resonance) while allowing the rest to pass.
            Lowpass12Db = FMOD_DSP_MULTIBAND_EQ_FILTER_LOWPASS_12DB,
            /// Resonant low-pass filter, attenuates frequencies (24dB per octave) above
            /// a given point (with specificed resonance) while allowing the rest to pass.
            Lowpass24Db = FMOD_DSP_MULTIBAND_EQ_FILTER_LOWPASS_24DB,
            /// Resonant low-pass filter, attenuates frequencies (48dB per octave) above
            /// a given point (with specificed resonance) while allowing the rest to pass.
            Lowpass48Db = FMOD_DSP_MULTIBAND_EQ_FILTER_LOWPASS_48DB,
            /// Resonant high-pass filter, attenuates frequencies (12dB per octave) below
            /// a given point (with specificed resonance) while allowing the rest to pass.
            Highpass12Db = FMOD_DSP_MULTIBAND_EQ_FILTER_HIGHPASS_12DB,
            /// Resonant high-pass filter, attenuates frequencies (24dB per octave) below
            /// a given point (with specificed resonance) while allowing the rest to pass.
            Highpass24Db = FMOD_DSP_MULTIBAND_EQ_FILTER_HIGHPASS_24DB,
            /// Resonant high-pass filter, attenuates frequencies (48dB per octave) below
            /// a given point (with specificed resonance) while allowing the rest to pass.
            Highpass48Db = FMOD_DSP_MULTIBAND_EQ_FILTER_HIGHPASS_48DB,
            /// Low-shelf filter, boosts or attenuates frequencies (with specified gain)
            /// below a given point while allowing the rest to pass.
            LowShelf = FMOD_DSP_MULTIBAND_EQ_FILTER_LOWSHELF,
            /// High-shelf filter, boosts or attenuates frequencies (with specified gain)
            /// above a given point while allowing the rest to pass.
            HighShelf = FMOD_DSP_MULTIBAND_EQ_FILTER_HIGHSHELF,
            /// Peaking filter, boosts or attenuates frequencies (with specified gain) at
            /// a given point (with specificed bandwidth) while allowing the rest to pass.
            Peaking = FMOD_DSP_MULTIBAND_EQ_FILTER_PEAKING,
            /// Band-pass filter, allows frequencies at a given point (with specificed
            /// bandwidth) to pass while attenuating frequencies outside this range.
            BandPass = FMOD_DSP_MULTIBAND_EQ_FILTER_BANDPASS,
            /// Notch or band-reject filter, attenuates frequencies at a given point (with
            /// specificed bandwidth) while allowing frequencies outside this range to pass.
            Notch = FMOD_DSP_MULTIBAND_EQ_FILTER_NOTCH,
            /// All-pass filter, allows all frequencies to pass, but changes
            /// the phase response at a given point (with specified sharpness).
            AllPass = FMOD_DSP_MULTIBAND_EQ_FILTER_ALLPASS,
        }
    }
}

/// Normalize DSP parameter types.
///
/// Normalize amplifies the sound based on the maximum peaks within the signal.
/// For example if the maximum peaks in the signal were 50% of the bandwidth,
/// it would scale the whole sound by 2.
///
/// The lower threshold value makes the normalizer ignore peaks below a certain
/// point, to avoid over-amplification if a loud signal suddenly came in, and
/// also to avoid amplifying to maximum things like background hiss.
///
/// Because FMOD is a realtime audio processor, it doesn't have the luxury of
/// knowing the peak for the whole sound (ie it can't see into the future),
/// so it has to process data as it comes in.
///
/// To avoid very sudden changes in volume level based on small samples of new
/// data, FMOD fades towards the desired amplification which makes for smooth
/// gain control. The fadetime parameter can control this.
pub mod Normalize {
    use super::*;

    dsp_params! { Normalize =>
        /// Time to ramp the silence to full.
        pub struct FadeTime(FMOD_DSP_NORMALIZE_FADETIME): f32;
        /// Lower volume range threshold to ignore.
        pub struct Threshold(FMOD_DSP_NORMALIZE_THRESHOLD): f32;
        /// Maximum amplification allowed.
        pub struct MaxAmp(FMOD_DSP_NORMALIZE_MAXAMP): f32;
    }
}

/// Object based spatializer parameters.
///
/// Signal processed by this DSP will be sent to the global object mixer
/// (effectively a send), any DSP connected after this will receive silence.
///
/// For best results this DSP should be used with [`OutputType::WinSonic`] or
/// [`OutputType::Audio3d`] to get height spatialization. Playback with any
/// other output will result in fallback spatialization provided by
/// [`DspType::Pan`].
///
/// [`ObjectPan::OverrideRange`] defaults to true for backwards compatability.
pub mod ObjectPan {
    use super::*;

    dsp_params! { ObjectPan =>
        /// 3D Position.
        pub struct Position3d(FMOD_DSP_OBJECTPAN_3D_POSITION): Attributes3dMulti;
        /// 3D Roll-off Type.
        pub struct Rolloff3d(FMOD_DSP_OBJECTPAN_3D_ROLLOFF): Rolloff3dType;
        /// 3D Min Distance when [`OverrideRange`] is true.
        pub struct MinDistance3d(FMOD_DSP_OBJECTPAN_3D_MIN_DISTANCE): i32;
        /// 3D Max Distance when [`OverrideRange`] is true.
        pub struct MaxDistance3d(FMOD_DSP_OBJECTPAN_3D_MAX_DISTANCE): i32;
        /// 3D Extent Mode.
        pub struct ExtentMode3d(FMOD_DSP_OBJECTPAN_3D_EXTENT_MODE): ExtentMode3dType;
        /// 3D Sound Size.
        pub struct SoundSize3d(FMOD_DSP_OBJECTPAN_3D_SOUND_SIZE): f32;
        /// 3D Min Extent.
        pub struct MinExtent3d(FMOD_DSP_OBJECTPAN_3D_MIN_EXTENT): f32;
        /// Overall gain to allow FMOD to know the DSP is scaling the signal for virtualization purposes.
        pub struct OverallGain(FMOD_DSP_OBJECTPAN_OVERALL_GAIN): super::OverallGain;
        /// Output gain level.
        pub struct OutputGain(FMOD_DSP_OBJECTPAN_OUTPUTGAIN): f32;
        /// Attenuation Range when [`OverrideRange`] is false.
        pub struct AttenuationRange(FMOD_DSP_OBJECTPAN_ATTENUATION_RANGE): f32;
        /// Override Attenuation Range with [`MinDistance3d`] and [`MaxDistance3d`].
        pub struct OverrideRange(FMOD_DSP_OBJECTPAN_OVERRIDE_RANGE): bool;
    }

    // MAYBE: wrap MinDistance3d and MaxDistance3d with a single Distance3d?
}

/// Oscillator DSP parameter types.
pub mod Oscillator {
    use super::*;

    dsp_params! { Oscillator =>
        /// Waveform type.
        pub struct Type(FMOD_DSP_OSCILLATOR_TYPE): Waveform;
        /// Frequency of the tone. Does not affect the noise generator.
        pub struct Rate(FMOD_DSP_OSCILLATOR_RATE): f32;
    }

    #[repr(i32)]
    #[non_exhaustive]
    #[allow(missing_docs)]
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[derive(::bytemuck::Contiguous, ::bytemuck::NoUninit)]
    pub enum Waveform {
        #[default]
        Sine = 0,
        Square = 1,
        SawUp = 2,
        SawDown = 3,
        Triangle = 4,
        Noise = 5,
    }

    impl DspParamType for Waveform {
        fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &Self) -> Result {
            dsp.set_parameter::<i32>(index, *value as i32)
        }

        fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<Self> {
            match dsp.get_parameter::<i32>(index)? {
                0 => Ok(Self::Sine),
                1 => Ok(Self::Square),
                2 => Ok(Self::SawUp),
                3 => Ok(Self::SawDown),
                4 => Ok(Self::Triangle),
                5 => Ok(Self::Noise),
                _ => Err(Error::InvalidParam),
            }
        }

        fn get_dsp_parameter_string<'a>(
            dsp: &Dsp,
            index: i32,
            bytes: &'a mut [u8],
        ) -> Result<&'a str> {
            i32::get_dsp_parameter_string(dsp, index, bytes)
        }
    }
}

/// Pan DSP parameter types.
///
/// FMOD_DSP_PAN_3D_PAN_BLEND controls the percentage of the effect supplied by FMOD_DSP_PAN_2D_DIRECTION and FMOD_DSP_PAN_2D_EXTENT.
///
/// For FMOD_DSP_PAN_3D_POSITION, the following members in the FMOD_DSP_PARAMETER_3DATTRIBUTES_MULTI struct should be non zero.
/// - numlisteners - This is typically 1, can be up to 8. Typically more than 1 is only used for split screen purposes. The FMOD Panner will average angles and produce the best compromise for panning and attenuation.
/// - relative[listenernum].position - This is the delta between the listener position and the sound position. Typically the listener position is subtracted from the sound position.
/// - relative[listenernum].forward - This is the sound's forward vector. Optional, set to 0,0,1 if not needed. This is only relevant for more than mono sounds in 3D, that are spread amongst the destination speakers at the time of panning.
///
///     If the sound rotates then the L/R part of a stereo sound will rotate amongst its destination speakers.
///     If the sound has moved and pinpointed into a single speaker, rotation of the sound will have no effect as at that point the channels are collapsed into a single point.
///
/// For FMOD_DSP_PAN_2D_STEREO_MODE, when it is set to FMOD_DSP_PAN_2D_STEREO_MODE_DISCRETE, only FMOD_DSP_PAN_2D_STEREO_SEPARATION and FMOD_DSP_PAN_2D_STEREO_AXIS are used.
/// When it is set to FMOD_DSP_PAN_2D_STEREO_MODE_DISTRIBUTED, then standard FMOD_DSP_PAN_2D_DIRECTION/FMOD_DSP_PAN_2D_EXTENT parameters are used.
///
/// FMOD_DSP_OBJECTPAN_OVERRIDE_RANGE defaults to true for backwards compatability.
pub mod Pan {
    use super::*;

    dsp_params! { Pan =>
        /// Panner mode.
        pub struct Mode(FMOD_DSP_PAN_MODE): ModeType;
        /// 2D Stero pan psition.
        pub struct StereoPosition2d(FMOD_DSP_PAN_2D_STEREO_POSITION): f32;
        /// 2D Surround pan direction. Direction from center point of panning circle where 0 is front center and -180 or +180 is rear speakers center point.
        pub struct Direction2d(FMOD_DSP_PAN_2D_DIRECTION): f32;
        /// 2D Surround pan extent.
        pub struct Extent2d(FMOD_DSP_PAN_2D_EXTENT): f32;
        /// 2D Surround pan rotation.
        pub struct Rotation2d(FMOD_DSP_PAN_2D_ROTATION): f32;
        /// 2D Surround pan LFE level.
        pub struct LfeLevel2d(FMOD_DSP_PAN_2D_LFE_LEVEL): f32;
        /// Stereo-To-Surround Mode.
        pub struct StereoMode2d(FMOD_DSP_PAN_2D_STEREO_MODE): StereoModeType2d;
        /// Stereo-To-Surround Stereo For [`Stereo2dModeType::Discrete`] mode.
        /// Separation/width of L/R parts of stereo sound.
        pub struct StereoSeparation2d(FMOD_DSP_PAN_2D_STEREO_SEPARATION): f32;
        /// Stereo-To-Surround Stereo For [`Stereo2dModeType::Discrete`] mode.
        /// Axis/rotation of L/R parts of stereo sound.
        pub struct StereoAxis2d(FMOD_DSP_PAN_2D_STEREO_AXIS): f32;
        /// Speakers Enabled Bitmask for each speaker from 0 to 32 to be considered by panner.
        /// Use to disable speakers from being panned to. 0 to 0xFFF. Default = 0xFFF (All on).
        pub struct EnabledSpeakers(FMOD_DSP_PAN_ENABLED_SPEAKERS): i32;
        /// 3D Position of panner and listener(s).
        pub struct Position3d(FMOD_DSP_PAN_3D_POSITION): Attributes3dMulti;
        /// 3D volume attenuation curve shape.
        pub struct Rolloff3d(FMOD_DSP_PAN_3D_ROLLOFF): Rolloff3dType;
        /// 3D volume attenuation minimum distance when [`OverrideRange`] is true.
        pub struct MinDistance3d(FMOD_DSP_OBJECTPAN_3D_MIN_DISTANCE): i32;
        /// 3D volume attenuation maximum distance when [`OverrideRange`] is true.
        pub struct MaxDistance3d(FMOD_DSP_OBJECTPAN_3D_MAX_DISTANCE): i32;
        /// 3D Extent Mode.
        pub struct ExtentMode3d(FMOD_DSP_OBJECTPAN_3D_EXTENT_MODE): ExtentMode3dType;
        /// 3D Sound Size.
        pub struct SoundSize3d(FMOD_DSP_OBJECTPAN_3D_SOUND_SIZE): f32;
        /// 3D Min Extent.
        pub struct MinExtent3d(FMOD_DSP_OBJECTPAN_3D_MIN_EXTENT): f32;
        /// 3D Pan Blend.
        pub struct PanBlend3d(FMOD_DSP_PAN_3D_PAN_BLEND): f32;
        /// LFE Upmix Enabled. Determines whether non-LFE source channels should
        /// mix to the LFE or leave it alone. 0 (off) to 1 (on). Default = 0 (off).
        pub struct LfeUpmixEnalbed(FMOD_DSP_PAN_LFE_UPMIX_ENABLED): i32;
        /// Overall gain to allow FMOD to know the DSP is scaling the signal for virtualization purposes.
        pub struct OverallGain(FMOD_DSP_OBJECTPAN_OVERALL_GAIN): super::OverallGain;
        /// Surround speaker mode.
        pub struct SurroundSpeakermode(FMOD_DSP_PAN_SURROUND_SPEAKER_MODE): SpeakerMode;
        /// 2D Height blend. When the input or [`SurroundSpeakerMode`] has height speakers,
        /// control the blend between ground and height. -1.0 (push top speakers to ground),
        /// 0.0 (preserve top / ground separation), 1.0 (push ground speakers to top).
        pub struct HeightBlend2d(FMOD_DSP_PAN_2D_HEIGHT_BLEND): f32;
        /// Attenuation Range when [`OverrideRange`] is false.
        pub struct AttenuationRange(FMOD_DSP_PAN_ATTENUATION_RANGE): f32;
        /// Override Attenuation Range with FMOD_DSP_PAN_3D_MIN_DISTANCE and FMOD_DSP_PAN_3D_MAX_DISTANCE.
        pub struct OverrideRange(FMOD_DSP_PAN_OVERRIDE_RANGE): bool;
    }

    // MAYBE: wrap MinDistance3d and MaxDistance3d with a single Distance3d?

    fmod_enum! {
        #[derive(Default)]
        /// Pan Mode values for Pan DSP.
        pub enum ModeType: FMOD_DSP_PAN_MODE_TYPE
        where const { self <= FMOD_DSP_PAN_MODE_SURROUND }
        {
            /// Single channel output.
            Mono = FMOD_DSP_PAN_MODE_MONO,
            /// Two channel output.
            Stereo = FMOD_DSP_PAN_MODE_STEREO,
            #[default]
            /// Three or more channel output. Includes common modes like quad, 5.1 or 7.1.
            Surround = FMOD_DSP_PAN_MODE_SURROUND,
        }
    }

    fmod_enum! {
        #[derive(Default)]
        /// 2D Stereo Mode values for Pan DSP.
        pub enum StereoModeType2d: FMOD_DSP_PAN_2D_STEREO_MODE_TYPE
        where const { self <= FMOD_DSP_PAN_2D_STEREO_MODE_DISCRETE }
        {
            /// The parts of a stereo sound are spread around destination speakers
            /// based on [`Extent2d`] / [`Direction2d`].
            Distributed = FMOD_DSP_PAN_2D_STEREO_MODE_DISTRIBUTED,
            #[default]
            /// The L/R parts of a stereo sound are rotated around a circle
            /// based on [`StereoAxis2d`] / [`StereoSeparation2d`].
            Discrete = FMOD_DSP_PAN_2D_STEREO_MODE_DISCRETE,
        }
    }
}

/// Parametric EQ DSP parameter types.
///
/// Parametric EQ is a single band peaking EQ filter that attenuates or
/// amplifies a selected frequency and its neighboring frequencies.
///
/// When the gain is set to zero decibels the sound will be unaffected and
/// represents the original signal exactly.
///
/// Deprecated and will be removed in a future release, to emulate with
/// [`DspType::MultibandEq`]:
///
/// ```rust,no_run
/// # let system = fmod::System::new()?;
/// # let multiband = system.create_dsp_by_type(fmod::DspType::MultibandEq)?;
/// # let center = 8000.0;
/// # let bandwidth = 1.0;
/// # let gain = 0.0;
/// // Configure a single band (band A) as a highpass (all other bands default to off).
/// // Center frequency can be used as with the old effect.
/// // Bandwidth can be applied by setting the 'Q' value of the new effect.
/// // Gain at the center frequency can be used the same as with the old effect.
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Filter, fmod::effect::MultibandEq::Filter::Peaking)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Frequency, center)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Q, bandwidth)?;
/// multiband.set_parameter(fmod::effect::MultibandEq::A::Gain, gain)?;
/// # Ok::<_, fmod::Error>(())
/// ```
#[deprecated = "Deprecated and will be removed in a future release."]
#[allow(deprecated)]
pub mod ParamEq {
    use super::*;

    dsp_params! { ParamEq =>
        /// Frequency center.
        pub struct Center(FMOD_DSP_PARAMEQ_CENTER): f32;
        /// Octave range around the center frequency to filter.
        pub struct Bandwidth(FMOD_DSP_PARAMEQ_BANDWIDTH): f32;
        /// Frequency Gain in dB.
        pub struct Gain(FMOD_DSP_PARAMEQ_GAIN): f32;
    }
}

/// Pitch shift DSP parameter types.
///
/// [`PitchShift::MaxChannels`] dictates the amount of memory allocated. By
/// default, the max_channels value is 0. If FMOD is set to stereo, the pitch
/// shift unit will allocate enough memory for 2 channels. If it is 5.1,
/// it will allocate enough memory for a 6 channel pitch shift, etc.
///
/// If the pitch shift effect is only ever applied to the global mix (i.e. with
/// [`ChannelControl::add_dsp`] on a [`ChannelGroup`] object), then 0 is the
/// value to set as it will be enough to handle all speaker modes.
///
/// When the pitch shift is added to a [`Channel`] (i.e. with
/// [`ChannelControl::add_dsp`] on a [`Channel`] object) then the signal channel
/// count that comes in could be anything from 1 to 8 possibly. It is only in
/// this case where you might want to increase the channel count above the
/// output's channel count.
///
/// If a [`Channel`] pitch shift is set to a lower number than the signal's
/// channel count that is coming in, it will not pitch shift the sound.
pub mod PitchShift {
    use super::*;

    dsp_params! { PitchShift =>
        /// Pitch value. 0.5 = one octave down, 2.0 = one octave up.
        /// 1.0 does not change the pitch.
        pub struct Pitch(FMOD_DSP_PITCHSHIFT_PITCH): f32;
        /// FFT window size - 256, 512, 1024, 2048, 4096. Increase this to reduce 'smearing'.
        /// This effect is a warbling sound similar to when an mp3 is encoded at very low bitrates.
        pub struct FftSize(FMOD_DSP_PITCHSHIFT_FFTSIZE): i32;
        /// Maximum channels supported. 0 = same as FMOD's default output
        /// polyphony, 1 = mono, 2 = stereo etc. See remarks for more.
        /// It is recommended to leave it at 0.
        pub struct MaxChannels(FMOD_DSP_PITCHSHIFT_MAXCHANNELS): i32;
    }
}

/// Return DSP parameter types.
pub mod Return {
    use super::*;

    dsp_params! { Return =>
        /// ID of this Return DSP.
        pub struct Id(FMOD_DSP_RETURN_ID): i32;
        /// Input speaker mode of this return.
        pub struct InputSpeakerMode(FMOD_DSP_RETURN_INPUT_SPEAKER_MODE): SpeakerMode;
    }
}

/// Send DSP parameter types.
pub mod Send {
    use super::*;

    dsp_params! { Send =>
        /// ID of the Return DSP this send is connected to where -1 indicates no connected return DSP.
        pub struct ReturnId(FMOD_DSP_SEND_RETURNID): i32;
        /// Send level.
        pub struct Level(FMOD_DSP_SEND_LEVEL): f32;
    }
}

/// SFX Reverb DSP parameter types.
///
/// This is a high quality I3DL2 based reverb. On top of the I3DL2 property set,
/// "Dry Level" is also included to allow the dry mix to be changed. These
/// properties can be set with presets available as associated constants on
/// [ReverbProperties](ReverbProperties#impl-ReverbProperties-1).
pub mod SfxReverb {
    use super::*;

    dsp_params! { SfxReverb =>
        /// Reverberation decay time at low-frequencies.
        pub struct DecayTime(FMOD_DSP_SFXREVERB_DECAYTIME): f32;
        /// Delay time of first reflection.
        pub struct EarlyDelay(FMOD_DSP_SFXREVERB_EARLYDELAY): f32;
        /// Late reverberation delay time relative to first reflection in milliseconds.
        pub struct LateDelay(FMOD_DSP_SFXREVERB_LATEDELAY): f32;
        /// Reference frequency for high-frequency decay.
        pub struct HfReference(FMOD_DSP_SFXREVERB_HFREFERENCE): f32;
        /// High-frequency decay time relative to decay time.
        pub struct HfDecayRatio(FMOD_DSP_SFXREVERB_HFDECAYRATIO): f32;
        /// Reverberation diffusion (echo density).
        pub struct Diffusion(FMOD_DSP_SFXREVERB_DIFFUSION): f32;
        /// Reverberation density (modal density).
        pub struct Density(FMOD_DSP_SFXREVERB_DENSITY): f32;
        /// Transition frequency of low-shelf filter.
        pub struct LowShelfFrequency(FMOD_DSP_SFXREVERB_LOWSHELFFREQUENCY): f32;
        /// Gain of low-shelf filter.
        pub struct LowShelfGain(FMOD_DSP_SFXREVERB_LOWSHELFGAIN): f32;
        /// Cutoff frequency of low-pass filter.
        pub struct HighCut(FMOD_DSP_SFXREVERB_HIGHCUT): f32;
        /// Blend ratio of late reverb to early reflections.
        pub struct EarlyLateMix(FMOD_DSP_SFXREVERB_EARLYLATEMIX): f32;
        /// Reverb signal level.
        pub struct WetLevel(FMOD_DSP_SFXREVERB_WETLEVEL): f32;
        /// Dry signal level.
        pub struct DryLevel(FMOD_DSP_SFXREVERB_DRYLEVEL): f32;
    }
}

/// Three EQ DSP parameter types.
pub mod ThreeEq {
    use super::*;

    dsp_params! { ThreeEq =>
        /// Low frequency gain.
        pub struct LowGain(FMOD_DSP_THREE_EQ_LOWGAIN): f32;
        /// Mid frequency gain.
        pub struct MidGain(FMOD_DSP_THREE_EQ_MIDGAIN): f32;
        /// High frequency gain.
        pub struct HighGain(FMOD_DSP_THREE_EQ_HIGHGAIN): f32;
        /// Low-to-mid crossover frequency.
        pub struct LowCrossover(FMOD_DSP_THREE_EQ_LOWCROSSOVER): f32;
        /// Mid-to-high crossover frequency.
        pub struct HighCrossover(FMOD_DSP_THREE_EQ_HIGHCROSSOVER): f32;
        /// Crossover Slope type.
        pub struct CrossoverSlope(FMOD_DSP_THREE_EQ_CROSSOVERSLOPE): f32;
    }
}

/// Transceiver DSP parameter types.
///
/// The transceiver only transmits and receives to a global array of 32 channels.
/// The transceiver can be set to receiver mode (like a return) and can receive
/// the signal at a variable gain. The transceiver can also be set to transmit
/// to a channel (like a send) and can transmit the signal with a variable gain.
///
/// The [`Transceiver::TransmitSpeakerMode`] is only applicable to the
/// transmission format, not the receive format. This means this parameter is
/// ignored in 'receive mode'. This allows receivers to receive at the speaker
/// mode of the user's choice. Receiving from a mono channel, is cheaper than
/// receiving from a surround channel for example. The 3 speaker modes
/// [`Transceiver::SpeakerMode::Mono`], [`Transceiver::SpeakerMode::Stereo`],
/// [`Transceiver::SpeakerMode::Surround`] are stored as separate buffers in
/// memory for a transmitter channel. To save memory, use 1 common speaker mode
/// for a transmitter.
///
/// The transceiver is double buffered to avoid desyncing of transmitters and
/// receivers. This means there will be a 1 block delay on a receiver, compared
/// to the data sent from a transmitter. Multiple transmitters sending to the
/// same channel will be mixed together.
pub mod Transceiver {
    use super::*;

    dsp_params! { Transceiver =>
        /// FALSE = Transceiver is a 'receiver' (like a return) and accepts data from a channel. TRUE = Transceiver is a 'transmitter' (like a send).
        pub struct Transmit(FMOD_DSP_TRANSCEIVER_TRANSMIT): bool;
        /// Gain to receive or transmit.
        pub struct Gain(FMOD_DSP_TRANSCEIVER_GAIN): f32;
        /// Global slot that can be transmitted to or received from.
        pub struct Channel(FMOD_DSP_TRANSCEIVER_CHANNEL): i32;
        /// Speaker mode (transmitter mode only).
        pub struct TransmitSpeakerMode(FMOD_DSP_TRANSCEIVER_TRANSMITSPEAKERMODE): SpeakerMode;
    }

    fmod_enum! {
        /// Speaker mode values for Transceiver DSP.
        ///
        /// The speaker mode of a transceiver buffer (of which there are up to 32 of)
        /// is determined automatically depending on the signal flowing through the
        /// transceiver effect, or it can be forced. Use a smaller fixed speaker mode
        /// buffer to save memory. Only relevant for transmitter dsps, as they control
        /// the format of the transceiver channel's buffer.
        ///
        /// If multiple transceivers transmit to a single buffer in different speaker
        /// modes, it will allocate memory for each speaker mode. This uses more memory
        /// than a single speaker mode. If there are multiple receivers reading from a
        /// channel with multiple speaker modes, it will read them all and mix them together.
        ///
        /// If the system's speaker mode is stereo or mono, it will not create a
        /// 3rd buffer, it will just use the mono/stereo speaker mode buffer.
        pub enum SpeakerMode: FMOD_DSP_TRANSCEIVER_SPEAKERMODE
        where
            const { self <= FMOD_DSP_TRANSCEIVER_SPEAKERMODE_SURROUND },
            const { self >= FMOD_DSP_TRANSCEIVER_SPEAKERMODE_AUTO },
        {
            /// A transmitter will use whatever signal channel count coming in to the transmitter,
            /// to determine which speaker mode is allocated for the transceiver channel.
            Auto = FMOD_DSP_TRANSCEIVER_SPEAKERMODE_AUTO,
            /// A transmitter will always downmix to a mono channel buffer.
            Mono = FMOD_DSP_TRANSCEIVER_SPEAKERMODE_MONO,
            /// A transmitter will always upmix or downmix to a stereo channel buffer.
            Stereo = FMOD_DSP_TRANSCEIVER_SPEAKERMODE_STEREO,
            /// A transmitter will always upmix or downmix to a surround channel buffer.
            /// Surround is the speaker mode of the system above stereo, so could be quad/surround/5.1/7.1.
            Surround = FMOD_DSP_TRANSCEIVER_SPEAKERMODE_SURROUND,
        }
    }
}

/// Tremolo DSP parameter types.
///
/// The tremolo effect varies the amplitude of a sound. Depending on the
/// settings, this unit can produce a tremolo, chopper or auto-pan effect.
///
/// The shape of the LFO (low freq. oscillator) can morphed between sine,
/// triangle and sawtooth waves using the [`Tremolo::Shape`] and
/// [`Tremolo::Skew`] parameters.
///
/// [`Tremolo::Duty`] and [`Tremolo::Square`] are useful for a chopper-type
/// effect where the first controls the on-time duration and second controls
/// the flatness of the envelope.
///
/// [`Tremolo::Spread`] varies the LFO phase between channels to get an
/// auto-pan effect. This works best with a sine shape LFO.
///
/// The LFO can be synchronized using the [`Tremolo::Phase`] parameter
/// which sets its instantaneous phase.
pub mod Tremolo {
    use super::*;

    dsp_params! { Tremolo =>
        /// LFO frequency.
        pub struct Frequency(FMOD_DSP_TREMOLO_FREQUENCY): f32;
        /// Tremolo depth.
        pub struct Depth(FMOD_DSP_TREMOLO_DEPTH): f32;
        /// LFO shape morph between triangle and sine.
        pub struct Shape(FMOD_DSP_TREMOLO_SHAPE): f32;
        /// Time-skewing of LFO cycle.
        pub struct Skew(FMOD_DSP_TREMOLO_SKEW): f32;
        /// LFO on-time.
        pub struct Duty(FMOD_DSP_TREMOLO_DUTY): f32;
        /// Flatness of the LFO shape.
        pub struct Square(FMOD_DSP_TREMOLO_SQUARE): f32;
        /// Instantaneous LFO phase.
        pub struct Phase(FMOD_DSP_TREMOLO_PHASE): f32;
        /// Rotation / auto-pan effect.
        pub struct Spread(FMOD_DSP_TREMOLO_SPREAD): f32;
    }
}

/// Side chain parameter data structure.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sidechain {
    /// Whether sidechains are enabled.
    pub sidechainenable: bool,
}

impl DspParamType for Sidechain {
    fn set_dsp_parameter(dsp: &Dsp, index: i32, value: &Self) -> Result {
        static_assert!(size_of::<FMOD_DSP_PARAMETER_SIDECHAIN>() == size_of::<FMOD_BOOL>());
        let value = value.sidechainenable as FMOD_BOOL;
        dsp.set_parameter::<[u8; size_of::<FMOD_BOOL>()]>(index, value.to_ne_bytes())
    }

    fn get_dsp_parameter(dsp: &Dsp, index: i32) -> Result<Self> {
        Ok(Self {
            sidechainenable: FMOD_BOOL::from_ne_bytes(dsp.get_parameter(index)?) != 0,
        })
    }

    fn get_dsp_parameter_string<'a>(dsp: &Dsp, index: i32, bytes: &'a mut [u8]) -> Result<&'a str> {
        <[u8; size_of::<FMOD_BOOL>()]>::get_dsp_parameter_string(dsp, index, bytes)
    }
}

fmod_struct! {
    /// Overall gain parameter data structure.
    ///
    /// This parameter is read by the system to determine the effect's gain for
    /// voice virtualization.
    pub struct OverallGain = FMOD_DSP_PARAMETER_OVERALLGAIN {
        /// Overall linear gain of the effect on the direct signal path.
        pub linear_gain: f32,
        /// Additive gain, for parallel signal paths.
        pub linear_gain_additive: f32,
    }
}

fmod_struct! {
    /// 3D attributes data structure for multiple listeners.
    ///
    /// The [`fmod::studio::System`] will set this parameter automatically if
    /// a [`fmod::studio::EventInstance`] position changes, however if using
    /// the core [`fmod::System`] you must set this DSP parameter explicitly.
    ///
    /// Attributes must use a coordinate system with the positive Y axis being
    /// up and the positive X axis being right. FMOD will convert passed in
    /// coordinates to left-handed for the plugin if the System was initialized
    /// with the [`InitFlags::Righthanded3d`] flag.
    ///
    /// When using a listener attenuation position, the direction of the
    /// `relative` attributes will be relative to the listener position and
    /// the length will be the distance to the attenuation position.
    pub struct Attributes3dMulti = FMOD_DSP_PARAMETER_3DATTRIBUTES_MULTI {
        /// Number of listeners.
        pub num_listeners: i32,
        /// Position of the sound relative to the listeners.
        pub relative: [Attributes3d; MAX_LISTENERS],
        /// Weighting of the listeners where 0 means listener has no contribution and 1 means full contribution.
        pub weight: [f32; MAX_LISTENERS],
        /// Position of the sound in world coordinates.
        pub absolute: Attributes3d,
    }
}

fmod_enum! {
    /// 3D roll-off values for Pan DSP.
    ///
    /// Minimum and Maximum distance settings are controlled with
    /// [`MinDistance3d`] and [`MaxDistance3d`].
    ///
    /// <style>main img { background-color: white; }</style>
    #[derive(Default)]
    pub enum Rolloff3dType: FMOD_DSP_PAN_3D_ROLLOFF_TYPE
    where const { self <= FMOD_DSP_PAN_3D_ROLLOFF_CUSTOM }
    {
        /// This is a linear-square roll-off model. Below `min_distance`, the volume
        /// is unattenuated; as distance increases from `min_distance` to `max_distance`,
        /// the volume attenuates to silence according to a linear squared gradient.
        /// For this roll-off mode, `distance` values greater than `min_distance` are
        /// scaled according to the [`rolloff_scale`](System::set_3d_settings). This
        /// roll-off mode provides steeper volume ramping close to the mindistance,
        /// and more gradual ramping close to the `max_distance`, than linear roll-off mode.
        ///
        /// ![Linear square roll-off](https://d1s9dnlmdewoh1.cloudfront.net/2.02/api/images/dsp-pan-3d-rolloff-linsquared.svg)
        LinearSquared = FMOD_DSP_PAN_3D_ROLLOFF_LINEARSQUARED,
        /// This is a linear roll-off model. Below `min_distance`, the volume
        /// is unattenuated; as distance increases from `min_distance` to `max_distance`,
        /// the volume attenuates to silence using a linear gradient.
        /// For this roll-off mode, `distance` values greater than `min_distance` are
        /// scaled according to the [`rolloff_scale`](System::set_3d_settings). While
        /// this roll-off mode is not as realistic as inverse roll-off mode,
        /// it is easier to comprehend.
        ///
        /// ![Linear roll-off](https://d1s9dnlmdewoh1.cloudfront.net/2.02/api/images/dsp-pan-3d-rolloff-linear.svg)
        Linear = FMOD_DSP_PAN_3D_ROLLOFF_LINEAR,
        #[default]
        /// This is an inverse roll-off model. Below `min_distance`, the volume
        /// is unattenuated; as distance increases above `min_distance`,
        /// the volume attenuates using mindistance/distance as the gradient until
        /// it reaches `max_distance`, where it stops attenuating.
        /// For this roll-off mode, `distance` values greater than `min_distance` are
        /// scaled according to the [`rolloff_scale`](System::set_3d_settings).
        /// This roll-off mode accurately models the way sounds attenuate over
        /// distance in the real world. (DEFAULT)
        ///
        /// ![Inverse roll-off](https://d1s9dnlmdewoh1.cloudfront.net/2.02/api/images/dsp-pan-3d-rolloff-inverse.svg)
        Inverse = FMOD_DSP_PAN_3D_ROLLOFF_INVERSE,
        /// This is a combination of the inverse and linear-square roll-off models.
        /// At short distances where inverse roll-off would provide greater attenuation,
        /// it functions as inverse roll-off mode; then at greater distances where linear-square
        /// roll-off mode would provide greater attenuation, it uses that roll-off mode instead.
        /// For this roll-off mode, `distance` values greater than `min_distance` are
        /// scaled according to the [`rolloff_scale`](System::set_3d_settings). Inverse
        /// tapered roll-off mode approximates realistic behavior while still guaranteeing
        /// the sound attenuates to silence at maxdistance.
        ///
        /// ![Inverse tapered roll-off](https://d1s9dnlmdewoh1.cloudfront.net/2.02/api/images/dsp-pan-3d-rolloff-invtaper.svg)
        InverseTapered = FMOD_DSP_PAN_3D_ROLLOFF_INVERSETAPERED,
        /// Custom roll-off can be defined by the programmer setting volume
        /// manually. Attenuation in the Pan DSP is turned off in this mode.
        Custom = FMOD_DSP_PAN_3D_ROLLOFF_CUSTOM,
    }
}

fmod_enum! {
    /// 3D Extent Mode values for Pan DSP.
    #[derive(Default)]
    #[allow(missing_docs)]
    pub enum ExtentMode3dType: FMOD_DSP_PAN_3D_EXTENT_MODE_TYPE
    where const { self <= FMOD_DSP_PAN_3D_EXTENT_MODE_OFF }
    {
        #[default]
        Auto = FMOD_DSP_PAN_3D_EXTENT_MODE_AUTO,
        User = FMOD_DSP_PAN_3D_EXTENT_MODE_USER,
        Off = FMOD_DSP_PAN_3D_EXTENT_MODE_OFF,
    }
}
