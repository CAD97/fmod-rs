/*============================================================================*/
//! Play Sound Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to simply load and play multiple sounds, the simplest
//! usage of FMOD. By default FMOD will decode the entire file into memory when
//! it loads. If the sounds are big and possibly take up a lot of RAM it would
//! be better to use the FMOD_CREATESTREAM flag, this will stream the file in
//! realtime as it plays.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use fmod_examples::{media, sleep_ms, yeet, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        let sound1 = system.create_sound(media!("drumloop.wav"), fmod::Mode::Default)?;

        // drumloop.wav has embedded loop points which automatically makes looping turn on,
        // so turn it off here. We could have also just put FMOD_LOOP_OFF in the above create_sound call.
        sound1.set_mode(fmod::Mode::LoopOff)?;

        let sound2 = system.create_sound(media!("jaguar.wav"), fmod::Mode::Default)?;
        let sound3 = system.create_sound(media!("swish.wav"), fmod::Mode::Default)?;

        // Main loop
        let mut channel = None;
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::Action1) {
                channel = Some(system.play_sound(&sound1, None)?);
            }

            if example.btn_press(Buttons::Action2) {
                channel = Some(system.play_sound(&sound2, None)?);
            }

            if example.btn_press(Buttons::Action3) {
                channel = Some(system.play_sound(&sound3, None)?);
            }

            system.update()?;

            let mut ms = 0;
            let mut lenms = 0;
            let mut playing = false;
            let mut paused = false;

            if let Some(channel) = channel {
                playing = match channel.is_playing() {
                    Ok(x) => x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => false,
                    Err(error) => yeet!(error),
                };
                paused = match channel.get_paused() {
                    Ok(x) => x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => false,
                    Err(error) => yeet!(error),
                };
                ms = match channel.get_position(fmod::TimeUnit::Ms) {
                    Ok(x) => x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => 0,
                    Err(error) => yeet!(error),
                };
                lenms = match channel.get_current_sound()? {
                    None => 0,
                    Some(current_sound) => match current_sound.get_length(fmod::TimeUnit::Ms) {
                        Ok(x) => x,
                        Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => 0,
                        Err(error) => yeet!(error),
                    },
                };
            }

            let channels_playing = system.get_channels_playing()?.all;

            example.draw("==================================================");
            example.draw("Play Sound Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to play a mono sound (drumloop)",
                Buttons::Action1.name(),
            ));
            example.draw(format_args!(
                "Press {} to play a mono sound (jaguar)",
                Buttons::Action2.name(),
            ));
            example.draw(format_args!(
                "Press {} to play a stereo sound (swish)",
                Buttons::Action3.name(),
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!(
                "Time {:02}:{:02}:{:02}/{:02}:{:02}:{:02} : {}",
                ms / 1000 / 60,
                ms / 1000 % 60,
                ms / 10 % 100,
                lenms / 1000 / 60,
                lenms / 1000 % 60,
                lenms / 10 % 100,
                if paused {
                    "Paused"
                } else if playing {
                    "Playing"
                } else {
                    "Stopped"
                }
            ));
            example.draw(format_args!("Channels Playing {channels_playing}"));

            sleep_ms(50);
        }

        // Shut down
        sound1.release()?;
        sound2.release()?;
        sound3.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
