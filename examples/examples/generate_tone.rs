/*============================================================================*/
//! Generate Tone Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to play generated tones using System::playDSP
//! instead of manually connecting and disconnecting DSP units.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use fmod_examples::{sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        // Create an oscillator DSP units for the tone.
        let dsp = system.create_dsp_by_type(fmod::DspType::Oscillator)?;
        dsp.set_parameter(fmod::effect::Oscillator::Rate, 440.0)?; // Musical note 'A'

        // Main loop
        let mut channel = None::<&fmod::Channel>;
        while !example.btn_press(Buttons::Quit) {
            example.update()?;
            system.update()?;

            if example.btn_press(Buttons::Action1) {
                if let Some(chan) = channel {
                    chan.stop()?;
                }

                let chan = system.create_dsp_channel(&dsp, None)?;
                channel = Some(chan);
                chan.set_volume(0.5)?;
                dsp.set_parameter(
                    fmod::effect::Oscillator::Type,
                    fmod::effect::Oscillator::Waveform::Sine,
                )?;
                chan.set_paused(false)?;
            }

            if example.btn_press(Buttons::Action2) {
                if let Some(chan) = channel {
                    chan.stop()?;
                }

                let chan = system.create_dsp_channel(&dsp, None)?;
                channel = Some(chan);
                chan.set_volume(0.5)?;
                dsp.set_parameter(
                    fmod::effect::Oscillator::Type,
                    fmod::effect::Oscillator::Waveform::Square,
                )?;
                chan.set_paused(false)?;
            }

            if example.btn_press(Buttons::Action3) {
                if let Some(chan) = channel {
                    chan.stop()?;
                }

                let chan = system.create_dsp_channel(&dsp, None)?;
                channel = Some(chan);
                chan.set_volume(0.5)?;
                dsp.set_parameter(
                    fmod::effect::Oscillator::Type,
                    fmod::effect::Oscillator::Waveform::SawUp,
                )?;
                chan.set_paused(false)?;
            }

            if example.btn_press(Buttons::Action4) {
                if let Some(chan) = channel {
                    chan.stop()?;
                }

                let chan = system.create_dsp_channel(&dsp, None)?;
                channel = Some(chan);
                chan.set_volume(0.5)?;
                dsp.set_parameter(
                    fmod::effect::Oscillator::Type,
                    fmod::effect::Oscillator::Waveform::Triangle,
                )?;
                chan.set_paused(false)?;
            }

            if example.btn_press(Buttons::More) {
                if let Some(chan) = channel {
                    chan.stop()?;
                    channel = None;
                }
            }

            if let Some(chan) = channel {
                if example.btn_down(Buttons::Up) || example.btn_down(Buttons::Down) {
                    let mut volume = chan.get_volume()?;
                    if example.btn_down(Buttons::Up) {
                        volume += 0.1;
                    } else {
                        volume -= 0.1;
                    };
                    volume = volume.clamp(0.0, 1.0);
                    chan.set_volume(volume)?;
                }

                if example.btn_down(Buttons::Left) || example.btn_down(Buttons::Right) {
                    let mut frequency = chan.get_frequency()?;
                    if example.btn_down(Buttons::Right) {
                        frequency += 500.0;
                    } else {
                        frequency -= 500.0;
                    };
                    chan.set_frequency(frequency)?;
                }
            }

            system.update()?;

            let (mut frequency, mut volume) = (0.0, 0.0);
            let mut playing = false;

            if let Some(chan) = channel {
                frequency = chan.get_frequency()?;
                volume = chan.get_volume()?;
                playing = chan.is_playing()?;
            }

            example.draw("==================================================");
            example.draw("Generate Tone Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to play a sine wave",
                Buttons::Action1.name()
            ));
            example.draw(format_args!(
                "Press {} to play a square wave",
                Buttons::Action2.name()
            ));
            example.draw(format_args!(
                "Press {} to play a saw wave",
                Buttons::Action3.name()
            ));
            example.draw(format_args!(
                "Press {} to play a triangle wave",
                Buttons::Action4.name()
            ));
            example.draw(format_args!(
                "Press {} to stop the channel",
                Buttons::More.name()
            ));
            example.draw(format_args!(
                "Press {} and {} to change volume",
                Buttons::Up.name(),
                Buttons::Down.name()
            ));
            example.draw(format_args!(
                "Press {} and {} to change frequency",
                Buttons::Left.name(),
                Buttons::Right.name()
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!(
                "Channel is {}",
                if playing { "playing" } else { "stopped" }
            ));
            example.draw(format_args!("Volume {:0.2}", volume));
            example.draw(format_args!("Frequency {:0.2}", frequency));

            sleep_ms(50);
        }

        // Shut down
        dsp.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
