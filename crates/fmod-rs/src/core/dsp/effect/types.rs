use {
    fmod::{raw::*, *},
    std::{mem::MaybeUninit, ptr, slice},
    zerocopy::{FromBytes, Immutable, IntoBytes},
};

fmod_enum! {
    /// DSP types.
    pub enum DspType: FMOD_DSP_TYPE {
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
        /// Three-band parametric equalizer. See [DspMultibandEq] for parameter
        /// information, [Effect reference - Multiband Equalizer] for overview.
        MultibandDynamics = FMOD_DSP_TYPE_MULTIBAND_DYNAMICS,
    }
}

/// A parameter index for a DSP effect.
pub trait DspParam: Into<i32> {
    /// The type of data this parameter is expecting.
    type Value: ?Sized + DspParamValue;
    /// The type of DSP this parameter is for. Mostly informational.
    const KIND: DspType;
}

/// A parameter index for a DSP effect that can be set.
pub trait DspParamMut: DspParam {}

#[allow(clippy::missing_safety_doc)]
pub(super) trait Sealed {}

/// Values that can be set for DSP parameters.
#[allow(private_bounds)]
#[allow(clippy::missing_safety_doc)]
pub unsafe trait DspParamValue: Sealed {
    #[doc(hidden)]
    unsafe fn set_dsp_parameter_unchecked(dsp: &Dsp, index: i32, value: &Self) -> Result;

    #[doc(hidden)]
    unsafe fn get_dsp_parameter_unchecked(dsp: &Dsp, index: i32) -> Result<Self>
    where
        Self: Sized,
    {
        let _ = (dsp, index);
        const { unreachable!() }
    }

    #[doc(hidden)]
    unsafe fn get_dsp_parameter_string_unchecked(
        dsp: &Dsp,
        index: i32,
        buf: &mut [MaybeUninit<u8>],
    ) -> Result;
}

impl Sealed for bool {}
unsafe impl DspParamValue for bool {
    unsafe fn set_dsp_parameter_unchecked(dsp: &Dsp, index: i32, value: &bool) -> Result {
        ffi!(FMOD_DSP_SetParameterBool(
            dsp.as_raw(),
            index,
            *value as FMOD_BOOL,
        ))?;
        Ok(())
    }

    unsafe fn get_dsp_parameter_unchecked(dsp: &Dsp, index: i32) -> Result<bool> {
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

    unsafe fn get_dsp_parameter_string_unchecked(
        dsp: &Dsp,
        index: i32,
        bytes: &mut [MaybeUninit<u8>],
    ) -> Result {
        ffi!(FMOD_DSP_GetParameterBool(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(())
    }
}

impl Sealed for i32 {}
unsafe impl DspParamValue for i32 {
    unsafe fn set_dsp_parameter_unchecked(dsp: &Dsp, index: i32, value: &i32) -> Result {
        ffi!(FMOD_DSP_SetParameterInt(dsp.as_raw(), index, *value))?;
        Ok(())
    }

    unsafe fn get_dsp_parameter_unchecked(dsp: &Dsp, index: i32) -> Result<i32> {
        let mut value = 0;
        ffi!(FMOD_DSP_GetParameterInt(
            dsp.as_raw(),
            index,
            &mut value,
            ptr::null_mut(),
            0,
        ))?;
        Ok(value)
    }

    unsafe fn get_dsp_parameter_string_unchecked(
        dsp: &Dsp,
        index: i32,
        bytes: &mut [MaybeUninit<u8>],
    ) -> Result {
        ffi!(FMOD_DSP_GetParameterInt(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(())
    }
}

impl Sealed for f32 {}
unsafe impl DspParamValue for f32 {
    unsafe fn set_dsp_parameter_unchecked(dsp: &Dsp, index: i32, value: &f32) -> Result {
        ffi!(FMOD_DSP_SetParameterFloat(dsp.as_raw(), index, *value))?;
        Ok(())
    }

    unsafe fn get_dsp_parameter_unchecked(dsp: &Dsp, index: i32) -> Result<f32> {
        let mut value = 0.0;
        ffi!(FMOD_DSP_GetParameterFloat(
            dsp.as_raw(),
            index,
            &mut value,
            ptr::null_mut(),
            0,
        ))?;
        Ok(value)
    }

    unsafe fn get_dsp_parameter_string_unchecked(
        dsp: &Dsp,
        index: i32,
        bytes: &mut [MaybeUninit<u8>],
    ) -> Result {
        ffi!(FMOD_DSP_GetParameterFloat(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as i32,
        ))?;
        Ok(())
    }
}

/// Byte data values that can be set for DSP parameters.
///
/// Implementing this trait gives a blanket impl of [`DspParamValue`], and can
/// be done for any type which is representationally just byte data, validated
/// by [zerocopy]'s derives for [`FromBytes`], [`IntoBytes`], and [`Immutable`].
pub trait DspDataParamValue: DspParamValue + FromBytes + IntoBytes + Immutable {}

impl<T: ?Sized + DspDataParamValue> Sealed for T {}
unsafe impl<T: ?Sized + DspDataParamValue> DspParamValue for T {
    unsafe fn set_dsp_parameter_unchecked(dsp: &Dsp, index: i32, value: &T) -> Result {
        let bytes = value.as_bytes();
        ffi!(FMOD_DSP_SetParameterData(
            dsp.as_raw(),
            index,
            bytes.as_ptr().cast_mut().cast(),
            bytes.len() as _,
        ))?;
        Ok(())
    }

    unsafe fn get_dsp_parameter_unchecked(dsp: &Dsp, index: i32) -> Result<T>
    where
        T: Sized,
    {
        let mut data = ptr::null_mut();
        let mut len = 0;
        ffi!(FMOD_DSP_GetParameterData(
            dsp.as_raw(),
            index,
            &mut data,
            &mut len,
            ptr::null_mut(),
            0,
        ))?;
        let bytes = unsafe { slice::from_raw_parts(data.cast(), len as _) };
        T::read_from_bytes(bytes).map_err(|_| Error::InvalidParam)
    }

    unsafe fn get_dsp_parameter_string_unchecked(
        dsp: &Dsp,
        index: i32,
        bytes: &mut [MaybeUninit<u8>],
    ) -> Result {
        ffi!(FMOD_DSP_GetParameterData(
            dsp.as_raw(),
            index,
            ptr::null_mut(),
            ptr::null_mut(),
            bytes.as_mut_ptr().cast(),
            bytes.len() as _,
        ))?;
        Ok(())
    }
}

impl<const N: usize> DspDataParamValue for [u8; N] {}
impl<const N: usize> DspDataParamValue for [i8; N] {}
impl<const N: usize> DspDataParamValue for [u16; N] {}
impl<const N: usize> DspDataParamValue for [i16; N] {}
impl<const N: usize> DspDataParamValue for [u32; N] {}
impl<const N: usize> DspDataParamValue for [i32; N] {}
impl<const N: usize> DspDataParamValue for [f32; N] {}
impl DspDataParamValue for [u8] {}
impl DspDataParamValue for [i8] {}
impl DspDataParamValue for [u16] {}
impl DspDataParamValue for [i16] {}
impl DspDataParamValue for [u32] {}
impl DspDataParamValue for [i32] {}
impl DspDataParamValue for [f32] {}
