(This document is from the FMOD API User Manual 2.02.06 (2022-03-14).
 &copy; 2022 Firelight Technologies Pty Ltd.)

## SDK version

FMOD is compiled using the following tools.

- **Xcode** - version 12.5.1 targeting macOS 11.3.

## Compatibility

FMOD supports x86_64 and arm64 back to macOS 10.9. Please note that both x86 and
PPC are not accepted for submission to the Mac App Store and thus are no longer
supported by FMOD.

## Libraries

### FMOD Core Engine library

- **/api/core/lib/libfmod.dylib** - Release binary for production code.
- **/api/core/lib/libfmodL.dylib** - Release binary with logging enabled for
  development.

FMOD.rs searches `$CARGO_MANIFEST_DIR/lib/$CARGO_CFG_TARGET_ARCH/` using Cargo
standard search path manipulation for `fmodL.dylib` or `fmod.dylib` depending on
whether `cfg(debug_assertions)` is true or false, respectively.

### FMOD Studio Engine library (used in conjunction with core library)

 - **/api/studio/lib/libfmodstudio.dylib** - Release binary for production code.
 - **/api/studio/lib/libfmodstudioL.dylib** - Release binary with logging
   enabled for development.

FMOD.rs searches `$CARGO_MANIFEST_DIR/lib/$CARGO_CFG_TARGET_ARCH/` using Cargo
standard search path manipulation for `fmodstudioL.dylib` or `fmodstudio.dylib`
depending on whether `cfg(debug_assertions)` is true or false, respectively.

## Latency

The default latency introduced by FMOD for this platform is 4 blocks of 512
samples at a sample rate of 48KHz, which equates to approximately 43ms. You are
free to change this using two APIs, [System::set_dsp_buffer_size] and
[System::set_software_format], but there are some important considerations.

All audio devices have a number of samples they prefer to operate in, on Mac
this is almost always 512, which makes our default a natural fit. If you use
[System::set_dsp_buffer_size] to reduce FMODs granularity (to 256 samples for
instance), be aware the audio device will still operate at its native block of
512 samples. If you would like to reduce the block size of the audio device (to
256 samples), after you have set the FMOD granularity and initialized the System
object use the following code:

```rust,ignore
let audio_unit = system.get_output_handle() as *mut AudioUnit;

let mut audio_device_id: AudioDeviceID = mem::zeroed();
let mut audio_device_id_size = mem::size_of::<AudioDeviceID>();
AudioUnitGetProperty(
    audio_unit,
    kAudioOutputUnitProperty_CurrentDevice,
    kAudioUnitScope_Global,
    0,
    &mut audio_device_id,
    &mut audio_device_id_size,
);

let mut bufer_frame_size: u32 = 256;
AudioDeviceSetProperty(
    audio_device_id,
    ptr::null(),
    0,
    false,
    kAudioDevicePropertyBufferFrameSize,
    mem::size_of::<u32>(),
    &mut buffer_frame_size,
);
```

FMOD.rs note: [`AudioDeviceSetProperty`] was deprecated in macOS 10.6 (2009,
Snow Leopard) and never exposed to Swift. This is probably a bad idea. Using
[`AudioUnitSetProperty`] or [`setPreferredIOBufferDuration`] may be preferable.

[`AudioDeviceSetProperty`]: https://developer.apple.com/documentation/coreaudio/1580742-audiodevicesetproperty?language=objc
[`AudioUnitSetProperty`]: https://developer.apple.com/documentation/audiotoolbox/1440371-audiounitsetproperty/
[`setPreferredIOBufferDuration`]: https://developer.apple.com/documentation/avfaudio/avaudiosession/1616589-setpreferrediobufferduration

## Thread Affinity

All threads will default to [ThreadAffinity::All]; it is not currently possible
to override this with [`thread::set_attributes`].

## Thread Priority

The relationship between FMOD platform agnostic thread priority and the platform specific values is as follows:

- [ThreadPriority::Low]       ~ 83
- [ThreadPriority::Medium]    ~ 87
- [ThreadPriority::High]      ~ 90
- [ThreadPriority::VeryHigh]  ~ 94
- [ThreadPriority::Extreme]   ~ 97
- [ThreadPriority::Critical]  ~ 99

# Performance Reference

This section is a companion for the [CPU Performance] white paper and serves as
a quick reference of facts targeting this platform.

[CPU Performance]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-cpu-performance.html

## Format Choice

Each compression format provided in FMOD has a reason for being included, the
below list will detail our recommendations for this platform. Formats listed as
primary are considering the best choice, secondary formats should only be
considered if the primary doesn't satisfy your requirements.

- **Vorbis**: Primary format for all sounds.
- **FADPCM**: Secondary format if Vorbis CPU usage is too high for low spec machines.
- **PCM**: Not recommended.
- **XMA**: Unavailable.
- **AT9**: Unavailable.

## Voice Count

To give developers an idea about the costs of a particular format we provide
synthetic benchmark results. These results are based on simple usage of the FMOD
Studio API using recommended configuration settings.

### Settings

- **Voice count:** 64
- **Sample rate:** 48KHz
- **Speaker mode:** Stereo
- **DSP block size:** 512 samples

## Test Device

- **CPU:** Intel(R) Core(TM) i5-4278U CPU @ 2.60GHz
- **OS:** macOS 10.15.7 (19H2)

### Results

- **DSP with Vorbis:** 7.78% (+/- 0.95%)
- **DSP with FADPCM:** 4.90% (+/- 0.61%)
- **DSP with PCM:** 2.93% (+/- 0.58%)
- **Update at 60 FPS:** 1.81% (+/- 0.35%)