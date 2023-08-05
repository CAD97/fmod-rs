/*============================================================================*/
//! Multiple System Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to play sounds on two different output devices from
//! the same application. It creates two fmod::System objects, selects a
//! different sound device for each, then allows the user to play one sound on
//! each device.
//!
//! Note that sounds created on device A cannot be played on device B and vice
//! versa.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

#![allow(clippy::try_err)]

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn fetch_driver(example: &mut Example, system: &fmod::System) -> anyhow::Result<i32> {
    let mut selected_index = 0;
    let num_drivers = system.get_num_drivers()?;

    if num_drivers == 0 {
        system.set_output(fmod::OutputType::NoSound)?;
    }

    example.update()?;
    let mut name = String::new();
    while !example.btn_press(Buttons::Action1) && !example.btn_press(Buttons::Quit) {
        example.update()?;

        if example.btn_press(Buttons::Up) && selected_index != 0 {
            selected_index -= 1;
        }
        if example.btn_press(Buttons::Down) && selected_index != num_drivers - 1 {
            selected_index += 1;
        }

        example.draw("==================================================");
        example.draw("Multiple System Example.");
        example.draw("Copyright (c) Firelight Technologies 2004-2023.");
        example.draw("==================================================");
        example.draw("");
        example.draw(format_args!("Choose a device for system: {:p}", system));
        example.draw("");
        example.draw(format_args!(
            "Use {} and {} to select.",
            Buttons::Up.name(),
            Buttons::Down.name()
        ));
        example.draw(format_args!(
            "Press {} to confirm.",
            Buttons::Action1.name()
        ));
        example.draw("");
        for i in 0..num_drivers {
            system.get_driver_name(i, &mut name)?;
            example.draw(format_args!(
                "[{}] - {i}. {name}",
                if selected_index == i { 'X' } else { ' ' }
            ));
        }

        sleep_ms(50);
    }

    Ok(selected_index)
}

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create Sound Card A
        let system_a = fmod::System::new()?;
        let driver = fetch_driver(&mut example, &system_a)?;
        system_a.set_driver(driver)?;
        system_a.init(32, fmod::InitFlags::Normal)?;

        // Create Sound Card B
        let system_b = unsafe { fmod::System::new_unchecked() }?;
        let driver = fetch_driver(&mut example, &system_b)?;
        system_b.set_driver(driver)?;
        system_b.init(32, fmod::InitFlags::Normal)?;

        // Load 1 sample into each soundcard.
        let sound_a = system_a.create_sound(media!("drumloop.wav"), fmod::Mode::LoopOff)?;
        let sound_b = system_b.create_sound(media!("jaguar.wav"), fmod::Mode::Default)?;

        // Main loop
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::Action1) {
                system_a.play_sound(&sound_a, None)?;
            }

            if example.btn_press(Buttons::Action2) {
                system_b.play_sound(&sound_b, None)?;
            }

            system_a.update()?;
            system_b.update()?;

            let channels_playing_a = system_a.get_channels_playing()?.all;
            let channels_playing_b = system_b.get_channels_playing()?.all;

            example.draw("==================================================");
            example.draw("Multiple System Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to play a sound on device A",
                Buttons::Action1.name()
            ));
            example.draw(format_args!(
                "Press {} to play a sound on device B",
                Buttons::Action2.name()
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!("Channels playing on A: {channels_playing_a}"));
            example.draw(format_args!("Channels playing on B: {channels_playing_b}"));

            sleep_ms(50);
        }

        // Shut down
        sound_a.release()?;
        system_a.release()?; // thread-unsafe

        sound_b.release()?;
        system_b.release()?;
    }

    example.close()?;

    Ok(())
}
