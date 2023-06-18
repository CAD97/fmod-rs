/*==============================================================================
//! Play Sound Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2021.
//!
//! This example shows how to simply load and play multiple sounds, the simplest
//! usage of FMOD. By default FMOD will decode the entire file into memory when
//! it loads. If the sounds are big and possibly take up a lot of RAM it would
//! be better to use the FMOD_CREATESTREAM flag, this will stream the file in
//! realtime as it plays.
==============================================================================*/

#![allow(clippy::try_err)]

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        let sound1 = system.create_sound(media!("drumloop.wav"), fmod::Mode::Default)?;

        // drumloop.wav has embedded loop points which automatically makes looping turn on,
        // so turn it off here.  We could have also just put FMOD_LOOP_OFF in the above CreateSound call.
        sound1.set_mode(fmod::Mode::LoopOff)?;

        let sound2 = system.create_sound(media!("jaguar.wav"), fmod::Mode::Default)?;
        let sound3 = system.create_sound(media!("swish.wav"), fmod::Mode::Default)?;

        let mut channel = None;
        while !example.btn_press(Buttons::Quit) {
            example.update()?;
            if example.btn_press(Buttons::Action1) {
                channel = Some(system.play_sound(&sound1, None, false)?);
            }
            if example.btn_press(Buttons::Action2) {
                channel = Some(system.play_sound(&sound2, None, false)?);
            }
            if example.btn_press(Buttons::Action3) {
                channel = Some(system.play_sound(&sound3, None, false)?);
            }
            system.update()?;

            let mut ms = 0;
            let mut lenms = 0;
            let mut playing = false;
            let mut paused = false;
            if let Some(channel) = channel.as_deref() {
                match channel.is_playing() {
                    Ok(x) => playing = x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {},
                    Err(error) => return Err(error)?,
                }
                match channel.get_paused() {
                    Ok(x) => paused = x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {},
                    Err(error) => return Err(error)?,
                }
                match channel.get_position(fmod::TimeUnit::Ms) {
                    Ok(x) => ms = x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {},
                    Err(error) => return Err(error)?,
                }
                match channel.get_current_sound() {
                    Ok(Some(current_sound)) => match current_sound.get_length(fmod::TimeUnit::Ms) {
                        Ok(x) => lenms = x,
                        Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {},
                        Err(error) => return Err(error)?,
                    },
                    Ok(None) | Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {},
                    Err(error) => return Err(error)?,
                }
            }

            let channels_playing = system.get_channels_playing()?.all;

            example.draw("==================================================");
            example.draw("Play Sound Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2022.");
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
    }

    example.close()?;

    Ok(())
}
