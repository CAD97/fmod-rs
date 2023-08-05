/*============================================================================*/
//! 3D Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to basic 3D positioning of sounds.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use fmod_examples::{media, sleep_ms, Buttons, Example};

/// 50ms update for interface
const INTERFACE_UPTIME: u64 = 50;

/// Units per meter. I.e feet would = 3.28. centimeters would = 100.
const DISTANCE_FACTOR: f32 = 1.0;

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;
        system.init(100, fmod::InitFlags::Normal)?;

        // Set the distance units. (meters/feet etc).
        system.set_3d_settings(fmod::Settings3d {
            doppler_scale: 1.0,
            distance_factor: DISTANCE_FACTOR,
            rolloff_scale: 1.0,
        })?;

        // Load some sounds
        let sound1 = system.create_sound(media!("drumloop.wav"), fmod::Mode::D3)?;
        sound1.set_3d_min_max_distance(0.5 * DISTANCE_FACTOR..5000.0 * DISTANCE_FACTOR)?;
        sound1.set_mode(fmod::Mode::LoopNormal)?;

        let sound2 = system.create_sound(media!("jaguar.wav"), fmod::Mode::D3)?;
        sound2.set_3d_min_max_distance(0.5 * DISTANCE_FACTOR..5000.0 * DISTANCE_FACTOR)?;
        sound2.set_mode(fmod::Mode::LoopNormal)?;

        let sound3 = system.create_sound(media!("swish.wav"), fmod::Mode::D2)?;

        // Play sounds at certain positions
        let channel1 = {
            let pos = fmod::Vector(-10.0 * DISTANCE_FACTOR, 0.0, 0.0);
            let vel = fmod::Vector(0.0, 0.0, 0.0);

            let channel = system.create_channel(&sound1, None)?;
            channel.set_3d_attributes(&pos, &vel)?;
            channel.set_paused(false)?;
            channel
        };

        let channel2 = {
            let pos = fmod::Vector(15.0 * DISTANCE_FACTOR, 0.0, 0.0);
            let vel = fmod::Vector(0.0, 0.0, 0.0);

            let channel = system.create_channel(&sound2, None)?;
            channel.set_3d_attributes(&pos, &vel)?;
            channel.set_paused(false)?;
            channel
        };

        // main loop
        let mut listenerflag = true;
        let mut listenerpos = fmod::Vector::new(0.0, 0.0, -1.0 * DISTANCE_FACTOR);
        let mut t = 0.0_f32;
        let mut lastpos = fmod::Vector::new(0.0, 0.0, 0.0);
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::Action1) {
                let paused = channel1.get_paused()?;
                channel1.set_paused(!paused)?;
            }

            if example.btn_press(Buttons::Action2) {
                let paused = channel2.get_paused()?;
                channel2.set_paused(!paused)?;
            }

            if example.btn_press(Buttons::Action3) {
                system.play_sound(&sound3, None)?;
            }

            if example.btn_press(Buttons::More) {
                listenerflag = !listenerflag;
            }

            if !listenerflag {
                if example.btn_down(Buttons::Left) {
                    listenerpos.x -= 1.0 * DISTANCE_FACTOR;
                    if (listenerpos.x) < -24.0 * DISTANCE_FACTOR {
                        listenerpos.x = -24.0 * DISTANCE_FACTOR;
                    }
                }

                if example.btn_down(Buttons::Right) {
                    listenerpos.x += 1.0 * DISTANCE_FACTOR;
                    if (listenerpos.x) > 23.0 * DISTANCE_FACTOR {
                        listenerpos.x = 23.0 * DISTANCE_FACTOR;
                    }
                }
            }

            // UPDATE THE LISTENER
            {
                let forward = fmod::Vector::new(0.0, 0.0, 1.0);
                let up = fmod::Vector::new(0.0, 1.0, 0.0);
                let mut vel = fmod::Vector::default();

                if listenerflag {
                    listenerpos.x = (t * 0.05).sin() * 24.0 * DISTANCE_FACTOR;
                    // left right pingpong
                }

                // ********* NOTE ******* READ NEXT COMMENT!!!!!
                // vel = how far we moved last FRAME (m/f), then time compensate it to SECONDS (m/s).
                vel.x = (listenerpos.x - lastpos.x) * (1000.0 / INTERFACE_UPTIME as f32);
                vel.y = (listenerpos.y - lastpos.y) * (1000.0 / INTERFACE_UPTIME as f32);
                vel.z = (listenerpos.z - lastpos.z) * (1000.0 / INTERFACE_UPTIME as f32);

                // store pos for next time
                lastpos = listenerpos;

                system.set_3d_listener_attributes(
                    0,
                    fmod::ListenerAttributes3d {
                        pos: listenerpos,
                        vel,
                        orientation: fmod::Orientation3d { forward, up },
                    },
                )?;

                t += 30.0 * (1.0 / INTERFACE_UPTIME as f32); // t is just a time value .. it increments in 30m/s steps in this example
            }

            system.update()?;

            // Create small visual display.
            let mut s = *b"|.............<1>......................<2>.......|";
            s[((listenerpos.x / DISTANCE_FACTOR) + 25.0) as usize] = b'L';

            example.draw("==================================================");
            example.draw("3D Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to toggle sound 1 (16bit Mono 3D)",
                Buttons::Action1.name(),
            ));
            example.draw(format_args!(
                "Press {} to toggle sound 2 (8bit Mono 3D)",
                Buttons::Action2.name(),
            ));
            example.draw(format_args!(
                "Press {} to play a sound (16bit Stereo 2D)",
                Buttons::Action3.name(),
            ));
            example.draw(format_args!(
                "Press {} or {} to move listener in still mode",
                Buttons::Left.name(),
                Buttons::Right.name(),
            ));
            example.draw(format_args!(
                "Press {} to toggle listener auto movement",
                Buttons::More.name(),
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(std::str::from_utf8(&s)?);

            sleep_ms(INTERFACE_UPTIME - 1);
        }
    }

    example.close()?;

    Ok(())
}
