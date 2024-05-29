#![allow(deprecated)]
/*============================================================================*/
//! Effects Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2024.
//!
//! This example shows how to apply some of the built in software effects to sounds
//! by applying them to the master channel group. All software sounds played here
//! would be filtered in the same way. To filter per channel, and not have other
//! channels affected, simply apply the same functions to the FMOD::Channel instead
//! of the FMOD::ChannelGroup.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;
        let master_group = system.get_master_channel_group()?;
        let sound = system.create_sound(media!("drumloop.wav"), fmod::Mode::Default)?;
        let channel = system.play_sound(&sound, None)?;

        // Create some effects to play with
        let dsp_lowpass = system.create_dsp_by_type(fmod::DspType::Lowpass)?;
        let dsp_highpass = system.create_dsp_by_type(fmod::DspType::Highpass)?;
        let dsp_echo = system.create_dsp_by_type(fmod::DspType::Echo)?;
        let dsp_flange = system.create_dsp_by_type(fmod::DspType::Flange)?;

        // Add them to the master channel group.  Each time an effect is added (to position 0) it pushes the others down the list.
        master_group.add_dsp(0, &dsp_lowpass)?;
        master_group.add_dsp(0, &dsp_highpass)?;
        master_group.add_dsp(0, &dsp_echo)?;
        master_group.add_dsp(0, &dsp_flange)?;

        // By default, bypass all effects.  This means let the original signal go through without processing.
        // It will sound 'dry' until effects are enabled by the user.
        dsp_lowpass.set_bypass(true)?;
        dsp_highpass.set_bypass(true)?;
        dsp_echo.set_bypass(true)?;
        dsp_flange.set_bypass(true)?;

        // Main loop
        while !example.btn_down(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::More) {
                let paused = channel.get_paused()?;
                channel.set_paused(!paused)?;
            }

            if example.btn_press(Buttons::Action1) {
                let bypass = dsp_lowpass.get_bypass()?;
                dsp_lowpass.set_bypass(!bypass)?;
            }

            if example.btn_press(Buttons::Action2) {
                let bypass = dsp_highpass.get_bypass()?;
                dsp_highpass.set_bypass(!bypass)?;
            }

            if example.btn_press(Buttons::Action3) {
                let bypass = dsp_echo.get_bypass()?;
                dsp_echo.set_bypass(!bypass)?;
            }

            if example.btn_press(Buttons::Action4) {
                let bypass = dsp_flange.get_bypass()?;
                dsp_flange.set_bypass(!bypass)?;
            }

            system.update()?;

            let dsp_lowpass_bypass = dsp_lowpass.get_bypass()?;
            let dsp_highpass_bypass = dsp_highpass.get_bypass()?;
            let dsp_echo_bypass = dsp_echo.get_bypass()?;
            let dsp_flange_bypass = dsp_flange.get_bypass()?;
            let paused = channel.get_paused()?;

            example.draw("==================================================");
            example.draw("Effects Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2024.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to pause/unpause sound",
                Buttons::More.name()
            ));
            example.draw(format_args!(
                "Press {} to toggle dsp lowpass effect",
                Buttons::Action1.name()
            ));
            example.draw(format_args!(
                "Press {} to toggle dsp highpass effect",
                Buttons::Action2.name()
            ));
            example.draw(format_args!(
                "Press {} to toggle dsp echo effect",
                Buttons::Action3.name()
            ));
            example.draw(format_args!(
                "Press {} to toggle dsp flange effect",
                Buttons::Action4.name()
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!(
                "{} : lowpass[{}] highpass[{}] echo[{}] flange[{}]",
                if paused { "Paused " } else { "Playing" },
                if dsp_lowpass_bypass { ' ' } else { 'x' },
                if dsp_highpass_bypass { ' ' } else { 'x' },
                if dsp_echo_bypass { ' ' } else { 'x' },
                if dsp_flange_bypass { ' ' } else { 'x' },
            ));

            sleep_ms(50);
        }

        // Shut down
        // TODO: sans remove_dsp, dropping the DSPs will cause a panic
        master_group.remove_dsp(&dsp_lowpass)?;
        master_group.remove_dsp(&dsp_highpass)?;
        master_group.remove_dsp(&dsp_echo)?;
        master_group.remove_dsp(&dsp_flange)?;

        dsp_lowpass.release()?;
        dsp_highpass.release()?;
        dsp_echo.release()?;
        dsp_flange.release()?;

        sound.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
