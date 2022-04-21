(This document is from the FMOD API User Manual 2.02.06 (2022-03-14).
 &copy; 2022 Firelight Technologies Pty Ltd.)

## SDK version

FMOD is compiled using the following tools.

- **Visual Studio 2019** - version 16.11.0 targeting platform toolset v142.

## Compatibility

FMOD supports the below architectures back to Windows 7.

- **x86** - optimized with SSE3.
- **x64** - optimized with SSE3 (and AVX if detected at runtime).

FMOD.rs names the architecture **x86_64**, not **x64**, matching Cargo.

## Libraries

*The provided libs are import libraries which require the corresponding DLL to*
*be present at runtime. Substitute $ARCH your desired architecture from the*
*'Compatibility' list above.*

The C API of the supplied libraries are compatible with MinGW (C++ ABI not
supported). The 64 bit dll can be linked directly. You will need to use the
import library libfmod.a and libfmodstudio.a in order to link the 32 bit dlls.

If you encounter issues linking fmod with MinGW, ensure that you are following
the GCC linker ordering requirements and the MinGW library search order.

### FMOD Core Engine library

- **/api/core/lib/$ARCH/fmod_vc.lib** - Release binary for production code
  (requires fmod.dll at runtime).
- **/api/core/lib/$ARCH/fmodL_vc.lib** - Release binary with logging enabled for
  development (requires fmodL.dll at runtime).

FMOD.rs searches `$CARGO_MANIFEST_DIR/lib/$CARGO_CFG_TARGET_ARCH/` using Cargo
standard search path manipulation for `fmodL.lib` or `fmod.lib` depending on
whether `cfg(debug_assertions)` is true or false, respectively.

### FMOD Studio Engine library (used in conjunction with core library)

- **/api/studio/lib/$ARCH/fmodstudio_vc.lib** - Release binary for production
  code (requires fmodstudio.dll at runtime).
- **/api/studio/lib/$ARCH/fmodstudioL_vc.lib** - Release binary with logging
  enabled for development (requires fmodstudioL.dll at runtime).

FMOD.rs searches `$CARGO_MANIFEST_DIR/lib/$CARGO_CFG_TARGET_ARCH/` using Cargo
standard search path manipulation for `fmodstudioL.lib` or `fmodstudio.lib`
depending on whether `cfg(debug_assertions)` is true or false, respectively.

## COM

Before calling any FMOD functions it is important to ensure COM is initialized.
You can achieve this by calling `CoInitializeEx(NULL, COINIT_APARTMENTTHREADED)`
on each thread that will interact with the FMOD API. This is balanced with a
call to `CoUninitialize()` when you are completely finished with all calls to
FMOD.

If you fail to initialize COM, FMOD will perform this on-demand for you issuing
a warning. FMOD will not uninitialize COM in this case so it will be considered
a memory leak.

To ensure correct behavior FMOD assumes when using the WASAPI output mode
(default for Windows Vista and newer) that you call [System::get_num_drivers],
[System::get_driver_info], and [System::init] from your UI thread. This ensures
that any platform specific dialogs that need to be presented can do so. This
recommendation comes from the [IAudioClient] interface docs on MSDN which state:

[IAudioClient]: https://docs.microsoft.com/en-us/windows/win32/api/audioclient/nn-audioclient-iaudioclient?redirectedfrom=MSDN

> **Note:** In Windows 8, the first use of IAudioClient to access the audio
> device should be on the STA thread. Calls from an MTA thread may result in
> undefined behavior.

FMOD.rs note: FMOD.rs does not handle COM initialization (it relies on FMOD's
default initialization and warning) nor does it attempt to restrict system init
to the UI thread.

## Thread Affinity

All threads will default to [ThreadAffinity::All]; this is recommended due to the
wide variety of PC hardware but can be customized with [thread::set_attributes].

## Thread Priority

The relationship between FMOD platform agnostic thread priority and the platform
specific values is as follows:

- [ThreadPriority::Low]       ~ THREAD_PRIORITY_BELOW_NORMAL
- [ThreadPriority::Medium]    ~ THREAD_PRIORITY_NORMAL
- [ThreadPriority::High]      ~ THREAD_PRIORITY_ABOVE_NORMAL
- [ThreadPriority::VeryHigh]  ~ THREAD_PRIORITY_HIGHEST
- [ThreadPriority::Extreme]   ~ THREAD_PRIORITY_TIME_CRITICAL
- [ThreadPriority::Critical]  ~ THREAD_PRIORITY_TIME_CRITICAL

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
- **DSP block size:** 1024 samples

### Test Device

- **CPU:** Intel(R) Core(TM) i7 CPU 860 @ 2.80GHz
- **OS:** Microsoft Windows [Version 6.1.7601]

### Results

- **DSP with Vorbis:** 6.88% (+/- 0.52%)
- **DSP with FADPCM:** 3.10% (+/- 0.20%)
- **DSP with PCM:** 1.59% (+/- 0.24%)
- **Update at 60 FPS:** 0.82% (+/- 0.05%)
