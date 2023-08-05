#![allow(unused)]

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        let sound = system.create_sound(media!("drumloop.wav"), fmod::Mode::Default)?;
        let channel = system.play_sound(&sound, None)?;

        // test whatever functionality here

        while !example.btn_press(Buttons::Quit) {
            example.update()?;
            sleep_ms(50);
        }
    }

    example.close()?;

    Ok(())
}
