/*============================================================================*/
//! Channel Groups Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2024.
//!
//! This example shows how to put channels into channel groups, so that you can
//! affect a group of channels at a time instead of just one.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        let sound0 = system.create_sound(media!("drumloop.wav"), fmod::Mode::LoopNormal)?;
        let sound1 = system.create_sound(media!("jaguar.wav"), fmod::Mode::LoopNormal)?;
        let sound2 = system.create_sound(media!("swish.wav"), fmod::Mode::LoopNormal)?;
        let sound3 = system.create_sound(media!("c.ogg"), fmod::Mode::LoopNormal)?;
        let sound4 = system.create_sound(media!("d.ogg"), fmod::Mode::LoopNormal)?;
        let sound5 = system.create_sound(media!("e.ogg"), fmod::Mode::LoopNormal)?;
        let sounds = [sound0, sound1, sound2, sound3, sound4, sound5];

        let group_a = system.create_channel_group(fmod::cstr8!("Group A"))?;
        let group_b = system.create_channel_group(fmod::cstr8!("Group B"))?;
        let master_group = system.get_master_channel_group()?;

        // Instead of being independent, set the group A and B to be children of the master group.
        master_group.add_group(&group_a)?;
        master_group.add_group(&group_b)?;

        // Start all the sounds.
        for (i, sound) in sounds.iter().enumerate() {
            let channel = system.create_sound_channel(sound, None)?;
            channel.set_channel_group(if i < 3 { &group_a } else { &group_b })?;
            channel.set_paused(false)?;
        }

        // Change the volume of each group, just because we can! (reduce overall noise).
        group_a.set_volume(0.5)?;
        group_b.set_volume(0.5)?;

        // Main loop.
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::Action1) {
                let mute = group_a.get_mute()?;
                group_a.set_mute(!mute)?;
            }

            if example.btn_press(Buttons::Action2) {
                let mute = group_b.get_mute()?;
                group_b.set_mute(!mute)?;
            }

            if example.btn_press(Buttons::Action3) {
                let mute = master_group.get_mute()?;
                master_group.set_mute(!mute)?;
            }

            system.update()?;

            let channels_playing = system.get_channels_playing()?.all;

            example.draw("==================================================");
            example.draw("Channel Groups Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2024.");
            example.draw("==================================================");
            example.draw("");
            example.draw("Group A : drumloop.wav, jaguar.wav, swish.wav");
            example.draw("Group B : c.ogg, d.ogg, e.ogg");
            example.draw("");
            example.draw(format_args!(
                "Press {} to mute/unmute group A",
                Buttons::Action1.name(),
            ));
            example.draw(format_args!(
                "Press {} to mute/unmute group B",
                Buttons::Action2.name(),
            ));
            example.draw(format_args!(
                "Press {} to mute/unmute master group",
                Buttons::Action3.name(),
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name(),));
            example.draw("");
            example.draw(format_args!("Channels playing {channels_playing}"));

            sleep_ms(50);
        }

        // A little fade out over 2 seconds.
        let mut pitch = 1.0;
        let mut vol = 1.0;
        for _ in 0..200 {
            master_group.set_pitch(pitch)?;
            master_group.set_volume(vol)?;

            vol -= 1.0 / 200.0;
            pitch -= 0.5 / 200.0;

            system.update()?;

            sleep_ms(10);
        }

        // Shut down.
        for sound in sounds {
            sound.release()?;
        }

        group_a.release()?;
        group_b.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
