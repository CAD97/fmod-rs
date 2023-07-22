#![allow(unused)]

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        let sound = system.create_sound(media!("drumloop.wav"), fmod::Mode::Default)?;
        let channel = system.play_sound(&sound, None, false)?;

        // test whatever functionality here
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Mono)?);
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Stereo)?);
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Quad)?);
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Surround)?);
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Surround51)?);
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Surround71)?);
        dbg!(system.get_speaker_mode_channels(fmod::SpeakerMode::Surround714)?);

        while !example.btn_press(Buttons::Quit) {
            example.update()?;
            sleep_ms(50);
        }
    }

    example.close()?;

    Ok(())
}
