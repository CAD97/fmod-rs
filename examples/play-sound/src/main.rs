#![allow(clippy::try_err)]

use fmod_example_framework::*;

fn main() -> anyhow::Result<()> {
    init()?;

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
        while !btn_press(Buttons::Quit) {
            update()?;
            if btn_press(Buttons::Action1) {
                channel = Some(system.play_sound(&*sound1, None, false)?);
            }
            if btn_press(Buttons::Action2) {
                channel = Some(system.play_sound(&*sound2, None, false)?);
            }
            if btn_press(Buttons::Action3) {
                channel = Some(system.play_sound(&*sound3, None, false)?);
            }
            system.update()?;

            let mut ms = 0;
            let mut lenms = 0;
            let mut playing = false;
            let mut paused = false;
            if let Some(channel) = channel {
                match channel.is_playing() {
                    Ok(x) => playing = x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {}
                    Err(error) => return Err(error)?,
                }
                match channel.get_paused() {
                    Ok(x) => paused = x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {}
                    Err(error) => return Err(error)?,
                }
                match channel.get_position(fmod::TimeUnit::Ms) {
                    Ok(x) => ms = x,
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {}
                    Err(error) => return Err(error)?,
                }
                match channel.get_current_sound() {
                    Ok(current_sound) => match current_sound.get_length(fmod::TimeUnit::Ms) {
                        Ok(x) => lenms = x,
                        Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {}
                        Err(error) => return Err(error)?,
                    },
                    Err(fmod::Error::InvalidHandle | fmod::Error::ChannelStolen) => {}
                    Err(error) => return Err(error)?,
                }
            }

            let channelsplaying = system.get_channels_playing()?;

            draw("==================================================");
            draw("Play Sound Example.");
            draw("Copyright (c) Firelight Technologies 2004-2021.");
            draw("==================================================");
            draw("");
            draw(format_args!(
                "Press {} to play a mono sound (drumloop)",
                btn_str(Buttons::Action1),
            ));
            draw(format_args!(
                "Press {} to play a mono sound (jaguar)",
                btn_str(Buttons::Action2),
            ));
            draw(format_args!(
                "Press {} to play a stereo sound (swish)",
                btn_str(Buttons::Action3),
            ));
            draw(format_args!("Press {} to quit", btn_str(Buttons::Quit),));
            draw("");
            draw(format_args!(
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
            draw(format_args!("Channels Playing {channelsplaying}"));

            sleep(50);
        }
    }

    close()?;

    Ok(())
}
