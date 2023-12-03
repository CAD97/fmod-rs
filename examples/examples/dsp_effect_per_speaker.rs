/*============================================================================*/
//! DSP Effect Per Speaker Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to manipulate a DSP network and as an example, creates 2
//! DSP effects, splitting a single sound into 2 audio paths, which it then filters
//! separately.
//!
//! To only have each audio path come out of one speaker each,
//! DSPConnection::setMixMatrix is used just before the 2 branches merge back together
//! again.
//!
//! For more speakers:
//!
//!  * Use System::setSoftwareFormat
//!  * Create more effects, currently 2 for stereo (lowpass and highpass), create one
//!    per speaker.
//!  * Under the 'Now connect the 2 effects to channeldsp head.' section, connect
//!    the extra effects by duplicating the code more times.
//!  * Filter each effect to each speaker by calling DSPConnection::setMixMatrix.  
//!    Expand the existing code by extending the matrices from 2 in and 2 out, to the
//!    number of speakers you require.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/
#![allow(deprecated)]

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;

        // In this special case we want to use stereo output and not worry about varying matrix sizes depending on user speaker mode.
        system.set_software_format(fmod::SoftwareFormat {
            sample_rate: 48000,
            speaker_mode: fmod::SpeakerMode::Stereo,
            num_raw_speakers: 0,
        })?;

        // Initialize FMOD
        system.init(32, fmod::InitFlags::Normal)?;
        let sound = system.create_sound(media!("drumloop.wav"), fmod::Mode::LoopNormal)?;
        let channel = system.play_sound(&sound, None)?;

        // Create the DSP effects.
        let dsp_lowpass = system.create_dsp_by_type(fmod::DspType::Lowpass)?;
        dsp_lowpass.set_parameter(fmod::effect::Lowpass::Cutoff, 1000.0)?;
        dsp_lowpass.set_parameter(fmod::effect::Lowpass::Resonance, 4.0)?;

        let dsp_highpass = system.create_dsp_by_type(fmod::DspType::Highpass)?;
        dsp_highpass.set_parameter(fmod::effect::Highpass::Cutoff, 4000.0)?;
        dsp_highpass.set_parameter(fmod::effect::Highpass::Resonance, 4.0)?;

        // Connect up the DSP network
        // When a sound is played, a subnetwork is set up in the DSP network which looks like this.
        // Wavetable is the drumloop sound, and it feeds its data from right to left.
        //
        // [DSPHEAD]<------------[DSPCHANNELMIXER]<------------[CHANNEL HEAD]<------------[WAVETABLE - DRUMLOOP.WAV]
        let master_group = system.get_master_channel_group()?;
        let dsp_head = unsafe { master_group.get_dsp_head()? };
        let (dsp_channel_mixer, _) = dsp_head.get_input(0)?;

        // Now disconnect channeldsp head from wavetable to look like this.
        //
        // [DSPHEAD]             [DSPCHANNELMIXER]<------------[CHANNEL HEAD]<------------[WAVETABLE - DRUMLOOP.WAV]
        dsp_head.disconnect_from_input(&dsp_channel_mixer)?;

        // Now connect the 2 effects to channeldsp head.
        // Store the 2 connections this makes so we can set their matrix later.
        //
        //          [DSPLOWPASS]
        //         /x
        // [DSPHEAD]             [DSPCHANNELMIXER]<------------[CHANNEL HEAD]<------------[WAVETABLE - DRUMLOOP.WAV]
        //         \y
        //          [DSPHIGHPASS]
        let dsp_lowpass_connection = // x = dsp_lowpass_connection
            dsp_head.add_input(&dsp_lowpass, fmod::DspConnectionType::Standard)?;
        let dsp_highpass_connection = // y = dsp_highpass_connection
            dsp_head.add_input(&dsp_highpass, fmod::DspConnectionType::Standard)?;
        let dsp_lowpass_connection = unsafe { dsp_lowpass_connection.as_ref() };
        let dsp_highpass_connection = unsafe { dsp_highpass_connection.as_ref() };

        // Now connect the channelmixer to the 2 effects
        //
        //          [DSPLOWPASS]
        //         /x          \
        // [DSPHEAD]             [DSPCHANNELMIXER]<------------[CHANNEL HEAD]<------------[WAVETABLE - DRUMLOOP.WAV]
        //         \y          /
        //          [DSPHIGHPASS]
        dsp_lowpass.add_input(&dsp_channel_mixer, fmod::DspConnectionType::Standard)?;
        dsp_highpass.add_input(&dsp_channel_mixer, fmod::DspConnectionType::Standard)?;
        // ignore connections - we don't care about them

        // Now the drumloop will be twice as loud, because it is being split into 2, then recombined at the end.
        // What we really want is to only feed the dsphead<-dsplowpass through the left speaker for that effect, and
        // dsphead<-dsphighpass to the right speaker for that effect.
        // We can do that simply by setting the pan, or speaker matrix of the connections.
        //
        //          [DSPLOWPASS]
        //         /x=1,0      \
        // [DSPHEAD]             [DSPCHANNELMIXER]<------------[CHANNEL HEAD]<------------[WAVETABLE - DRUMLOOP.WAV]
        //         \y=0,1      /
        //          [DSPHIGHPASS]
        let lowpass_matrix = [
            [1.0, 0.0], // <- output to front left.  Take front left input signal at 1.0.
            [0.0, 0.0], // <- output to front right.  Silence
        ];
        let highpass_matrix = [
            [0.0, 0.0], // <- output to front left.  Silence
            [0.0, 1.0], // <- output to front right.  Take front right input signal at 1.0
        ];

        // Upgrade the signal coming from the channel mixer from mono to stereo.  Otherwise the lowpass and highpass will get mono signals
        dsp_channel_mixer.set_channel_format(0, fmod::SpeakerMode::Stereo)?;

        // Now set the above matrices.
        dsp_lowpass_connection.set_mix_matrix(&lowpass_matrix)?;
        dsp_highpass_connection.set_mix_matrix(&highpass_matrix)?;

        dsp_lowpass.set_bypass(true)?;
        dsp_highpass.set_bypass(true)?;

        dsp_lowpass.set_active(true)?;
        dsp_highpass.set_active(true)?;

        // Main loop
        let mut pan = 0.0_f32;
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            let mut low_bypass = dsp_lowpass.get_bypass()?;
            let mut high_bypass = dsp_highpass.get_bypass()?;

            if example.btn_press(Buttons::Action1) {
                low_bypass = !low_bypass;
                dsp_lowpass.set_bypass(low_bypass)?;
            }

            if example.btn_press(Buttons::Action2) {
                high_bypass = !high_bypass;
                dsp_highpass.set_bypass(high_bypass)?;
            }

            if example.btn_down(Buttons::Left) {
                pan = (pan - 0.1).clamp(-1.0, 1.0);
                channel.set_pan(pan)?;
            }

            if example.btn_down(Buttons::Right) {
                pan = (pan + 0.1).clamp(-1.0, 1.0);
                channel.set_pan(pan)?;
            }

            system.update()?;

            example.draw("==================================================");
            example.draw("DSP Effect Per Speaker Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to toggle lowpass (left speaker)",
                Buttons::Action1.name()
            ));
            example.draw(format_args!(
                "Press {} to toggle highpass (right speaker)",
                Buttons::Action2.name()
            ));
            example.draw(format_args!(
                "Press {} or {} to pan sound",
                Buttons::Left.name(),
                Buttons::Right.name()
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!(
                "Lowpass (left) is {}",
                if low_bypass { "inactive" } else { "active" }
            ));
            example.draw(format_args!(
                "Highpass (right) is {}",
                if high_bypass { "inactive" } else { "active" }
            ));
            example.draw(format_args!("Pan is {:0.2}", pan));

            sleep_ms(50);
        }

        // Shut down
        sound.release()?;
        dsp_lowpass.release()?;
        dsp_highpass.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
