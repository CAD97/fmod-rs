# Platform Details | iOS

(This document is from the FMOD API User Manual 2.02.06 (2022-03-14). © 2022 Firelight Technologies Pty Ltd.)
<!-- This markdown is generated by docgen. Do not edit by hand. -->

  
<pre class="ignore" style="white-space:normal;font:inherit;">
FMOD.rs is not tested to work on or even compile for iOS.
</pre>

## iOS Specific Starter Guide

 ### SDK Version

 FMOD is compiled using the following tools.

  - **Xcode** - version 12.5.1 targeting SDK 14.5.
 
 ### Compatibility

 FMOD supports devices of the below architectures back to iOS / tvOS 9.0. Please note that armv6 is no longer accepted for submission to the iOS App Store and thus is no longer supported by FMOD.

  - **iOS**: armv7, armv7s, arm64, arm64e
 - **tvOS**: arm64, arm64e
 - **iOS simulator**: x86, x86_64, arm64
 - **tvOS simulator**: x86_64, arm64
 
 ### Libraries

 *Each lib is a universal binary containing the relevant architectures from the 'Compatibility' list above.*

 #### FMOD Core Engine library

  - **/api/core/lib/libfmod_iphoneos.a** - Release iOS device binary for production code.
 - **/api/core/lib/libfmodL_iphoneos.a** - Release iOS device binary with logging enabled for development.
 - **/api/core/lib/libfmod_iphonesimulator.a** - Release iOS simulator binary for production code.
 - **/api/core/lib/libfmodL_iphonesimulator.a** - Release iOS simulator binary with logging enabled for development.
 - **/api/core/lib/libfmod_appletvos.a** - Release tvOS device binary for production code.
 - **/api/core/lib/libfmodL_appletvos.a** - Release tvOS device binary with logging enabled for development.
 - **/api/core/lib/libfmod_appletvsimulator.a** - Release tvOS simulator binary for production code.
 - **/api/core/lib/libfmodL_appletvsimulator.a** - Release tvOS simulator binary with logging enabled for development.
 
 
<pre class="ignore" style="white-space:normal;font:inherit;">
FMOD.rs searches <code>$CARGO_MANIFEST_DIR/lib/$CARGO_CFG_TARGET_ARCH/</code>
using Cargo standard search path manipulation for <code>libfmodL.a</code> or
<code>libfmod.a</code> depending on whether <code>cfg(debug_assertions)</code>
is true or false, respectively.
</pre>

#### FMOD Studio Engine library (used in conjunction with core library)

  - **/api/studio/lib/libfmodstudio_iphoneos.a** - Release iOS device binary for production code.
 - **/api/studio/lib/libfmodstudioL_iphoneos.a** - Release iOS device binary with logging enabled for development.
 - **/api/studio/lib/libfmodstudio_iphonesimulator.a** - Release iOS simulator binary for production code.
 - **/api/studio/lib/libfmodstudioL_iphonesimulator.a** - Release iOS simulator binary with logging enabled for development.
 - **/api/studio/lib/libfmodstudio_appletvos.a** - Release tvOS device binary for production code.
 - **/api/studio/lib/libfmodstudioL_appletvos.a** - Release tvOS device binary with logging enabled for development.
 - **/api/studio/lib/libfmodstudio_appletvsimulator.a** - Release tvOS simulator binary for production code.
 - **/api/studio/lib/libfmodstudioL_appletvsimulator.a** - Release tvOS simulator binary with logging enabled for development.
 
 
<pre class="ignore" style="white-space:normal;font:inherit;">
FMOD.rs searches <code>$CARGO_MANIFEST_DIR/lib/$CARGO_CFG_TARGET_ARCH/</code>
using Cargo standard search path manipulation for <code>libfmodstudioL.a</code>
or <code>libfmodstudio.a</code> depending on whether
<code>cfg(debug_assertions)</code> is true or false, respectively.
</pre>

#### Apple libraries (required for Core and Studio Engine libraries)

  - **AudioToolbox.framework** - Audio output.
 - **AVFoundation.framework** - Audio Session query.
 
 
<pre class="ignore" style="white-space:normal;font:inherit;">
FMOD.rs does not link the above frameworks; it is your responsibility to ensure
that these frameworks are linked properly.
</pre>

### Hardware Decoding

 Via the AudioQueue codec FMOD supports decoding AAC, ALAC and MP3. At present iOS devices only have support for decoding one sound with hardware at a time (which may be consumed by playing the iPod). At the cost of extra CPU, all iOS devices have access to software codecs to support more than one sound of these formats. By default FMOD will try to use hardware, if it is in use a software codec will be used. If you want explicit control over whether hardware or software is chosen you can use the [`FMOD_AUDIOQUEUE_CODECPOLICY`](FMOD_AUDIOQUEUE_CODECPOLICY "Control whether the sound will use a the dedicated hardware decoder or a software codec.") enumeration provided in fmod_ios.h. This is set with [`CreateSoundExInfo::audioqueuepolicy`](CreateSoundExInfo::audioqueuepolicy "") via [`System::create_sound`](System::create_sound "Loads a sound into memory, opens it for streaming or sets it up for callback based sounds.").

 When playing MP3s using the AudioQueue codec, seeking is generally slow for the first time each position is visited. If you need fast random access to a file you can create the sound using the [`FMOD_ACCURATETIME`](FMOD_ACCURATETIME "") flag. This will scan the file at load time to determine its accurate length, which has the benefit of creating a seek table to aid in seeking. This is a one-time upfront cost for fast seeking vs paying the cost at runtime for each unique position.

 All decoding performed by the AudioQueue codec is done on standalone files such as .mp3, .m4a, etc. There is no support for using AudioQueue with FSB / Bank compressed audio, any MP3 decoding for FSB files will be performed by the standard cross-platform FMOD decoder.

 ### Handling Interruptions

 Unlike in previous versions of FMOD, it is now the responsibility of the developer to interact with the AudioSession APIs native to this platform. To assist in this matter we provide two functions you can use when you need to handle interruptions, [`System::mixer_suspend`](System::mixer_suspend "Suspend mixer thread and relinquish usage of audio hardware while maintaining internal state.") and [`System::mixer_resume`](System::mixer_resume "Resume mixer thread and reacquire access to audio hardware."). For more information about interruptions please check the [Apple documentation](<http://developer.apple.com/Library/ios/documentation/Audio/Conceptual/AudioSessionProgrammingGuide/HandlingAudioInterruptions/HandlingAudioInterruptions.html>).

``````````objective-c
[[NSNotificationCenter defaultCenter] addObserverForName:AVAudioSessionInterruptionNotification object:nil queue:nil usingBlock:^(NSNotification *notification)
{
    bool began = [[notification.userInfo valueForKey:AVAudioSessionInterruptionTypeKey] intValue] == AVAudioSessionInterruptionTypeBegan;

    if (began == gIsSuspended)
    {
        return;
    }
    if (@available(iOS 10.3, *))
    {
        if (began && [[notification.userInfo valueForKey:AVAudioSessionInterruptionWasSuspendedKey] boolValue])
        {
            return;
        }
    }

    gIsSuspended = began;
    if (!began)
    {
        [[AVAudioSession sharedInstance] setActive:TRUE error:nil];
    }
    if (gSuspendCallback)
    {
        gSuspendCallback(began);
    }
}];

[[NSNotificationCenter defaultCenter] addObserverForName:UIApplicationDidBecomeActiveNotification object:nil queue:nil usingBlock:^(NSNotification *notification)
{
#ifndef TARGET_OS_TV
    if (!gIsSuspended)
    {
        return;
    }
#else
    if (gSuspendCallback)
    {
        gSuspendCallback(true);
    }
#endif
    [[AVAudioSession sharedInstance] setActive:TRUE error:nil];
    if (gSuspendCallback)
    {
        gSuspendCallback(false);
    }
    gIsSuspended = false;
}];
``````````

 ### Lock Screen & Background Audio

 There is no special configuration inside FMOD required to enable the playback of audio from the lock screen or the background, there are two things you must configure outside of FMOD to do this though.

  1. Choose an AudioSession category that supports background / lock screen audio, see [audio session basics](<http://developer.apple.com/Library/ios/documentation/Audio/Conceptual/AudioSessionProgrammingGuide/AudioSessionBasics/AudioSessionBasics.html>) for more details.
 1. Enable background audio functionality in your info.plist with the UIBackgroundModes key, see the [iOS key reference](<http://developer.apple.com/Library/ios/documentation/General/Reference/InfoPlistKeyReference/Articles/iPhoneOSKeys.html>) for more details.
 
 When playing audio on the lock screen (or during the fade out transition to silence when locking) it is important to ensure your buffering is configured correctly to allow low power audio playback. Please consult the latency section of this doc for further details.

 ### Recording

 Much like lock screen and background audio, recording requires a particular AudioSession category to be active at the time of System::record_start (and must remain active until the recording finishes). The required category is called 'play and record' and can be read about in the [audio session basics](<http://developer.apple.com/Library/ios/documentation/Audio/Conceptual/AudioSessionProgrammingGuide/AudioSessionBasics/AudioSessionBasics.html>) documentation. Note that FMOD is always 'playing' audio (even silence) so it is not sufficient to simply use the 'recording' category unless you are running the 'No Sound' or 'Wav Writer' output mode.

 Some devices may take some time to switch AudioSession category so it is recommended to set this category at application start time to avoid any hiccups in audio playback.

 You will also need to add a "Privacy - Microphone Usage Description" ([NSMicrophoneUsageDescription](<https://developer.apple.com/documentation/bundleresources/information_property_list/nsmicrophoneusagedescription>)) key to the built project's Info.plist file, with a string value explaining to the user how your application will use their recorded data.

 ### Latency

 The default latency introduced by FMOD for this platform is 4 blocks of 512 samples at a sample rate of 24KHz, which equates to approximately 85ms. You are free to change this using two APIs, System::set_dsp_buffer_size and System::set_software_format but there are some important considerations.

 If you have configured background or lock screen audio when locking the device the OS will conserve power by requesting audio from FMOD less frequently. If you desire this functionality please ensure your DSP buffer size is sufficiently large to cover the request. The iOS operating system will expect 4096 samples to be available, so configure FMOD as 8 blocks of 512 samples or 4 blocks of 1024 samples to satisfy the request (otherwise silence will be produced and a warning issued on the TTY).

 If you are worried about latency and do not want automatic low power mode you can configure the Audio Session buffer and sample rate to match FMOD for best results. Assuming an FMOD block size of 512 samples and 24KHz sample rate you should configure the OS with the following:

``````````objective-c
AVAudioSession *session = [AVAudioSession sharedInstance];
double rate = 24000.0; // This should match System::set_software_format 'samplerate' which defaults to 24000
int blockSize = 512; // This should match System::set_dsp_buffer_size 'bufferlength' which defaults to 512

BOOL success = [session setPreferredSampleRate:rate error:nil];
assert(success);

success = [session setPreferredIOBufferDuration:blockSize / rate error:nil];
assert(success);

success = [session setActive:TRUE error:nil];
assert(success);
``````````

 ### Multichannel Output

 For hardware that supports greater than stereo output you can configure the device to operate with that channel count using the AudioSession API.

 Here is a code snippet that demonstrates using as many channels as available:

``````````objective-c
AVAudioSession *session = [AVAudioSession sharedInstance];
long maxChannels = [session maximumOutputNumberOfChannels];

BOOL success = [session setPreferredOutputNumberOfChannels:maxChannels error:nil];
assert(success);

success = [session setActive:TRUE error:nil];
assert(success);
``````````

 ### Thread Affinity

 All threads will default to [`ThreadAffinity::CoreAll`](ThreadAffinity::CoreAll ""), it is not currently possible to override this with [`raw::FMOD_Thread_SetAttributes`](raw::FMOD_Thread_SetAttributes "Specify the affinity, priority and stack size for all FMOD created threads.").

 ### Thread Priority

 The relationship between FMOD platform agnostic thread priority and the platform specific values is as follows:

  - [`ThreadPriority::Low`](ThreadPriority::Low "") ~ 83
 - [`ThreadPriority::Medium`](ThreadPriority::Medium "") ~ 87
 - [`ThreadPriority::High`](ThreadPriority::High "") ~ 90
 - [`ThreadPriority::VeryHigh`](ThreadPriority::VeryHigh "") ~ 94
 - [`ThreadPriority::Extreme`](ThreadPriority::Extreme "") ~ 97
 - [`ThreadPriority::Critical`](ThreadPriority::Critical "") ~ 99
 
 For FMOD to detect the channel count you must use setPreferredOutputNumberOfChannels and activate your AudioSession before calling [`System::init`](System::init "Initialize the system object and prepare FMOD for playback.").

 ## Performance Reference

 This section is a companion for the [CPU Performance](<https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-cpu-performance.html>) white paper and serves as a quick reference of facts targeting this platform.

 ### Format Choice

 Each compression format provided in FMOD has a reason for being included, the below list will detail our recommendations for this platform. Formats listed as primary are considering the best choice, secondary formats should only be considered if the primary doesn't satisfy your requirements.

  - **FADPCM**: Primary format for all sounds.
 - **Vorbis**: Secondary format for long streams if FADPCM compression is too low.
 - **PCM**: Secondary format for short sounds if FADPCM cost is too high.
 - **AAC**: Special format for long streams, single hardware assisted codec available for .MP4 / .M4A files. 
 - **XMA**: Unavailable.
 - **AT9**: Unavailable.
 
 ### Voice Count

 To give developers an idea about the costs of a particular format we provide synthetic benchmark results. These results are based on simple usage of the FMOD Studio API using recommended configuration settings.

 #### Settings

  - **Voice count:** 32
 - **Sample rate:** 24KHz
 - **Speaker mode:** Stereo
 - **DSP block size:** 1024 samples
 
 #### Test Device: A

  - **CPU:** Apple A7 @ 1.3 GHz (iPhone 5S)
 - **OS:** 12.5.1
 
 #### Results: A

  - **DSP with Vorbis:** 5.27% (+/- 0.76%)
 - **DSP with FADPCM:** 2.21% (+/- 0.30%)
 - **DSP with PCM:** 1.36% (+/- 0.23%)
 - **Update at 60 FPS:** 0.89% (+/- 0.52%)
 
 #### Test Device: B

  - **CPU:** Apple A8 @ 1.5 GHz (iPad mini 4)
 - **OS:** 14.4.1
 
 #### Results: B

  - **DSP with Vorbis:** 4.16% (+/- 0.76%)
 - **DSP with FADPCM:** 1.68% (+/- 0.50%)
 - **DSP with PCM:** 0.97% (+/- 0.42%)
 - **Update at 60 FPS:** 0.82% (+/- 1.01%)
 