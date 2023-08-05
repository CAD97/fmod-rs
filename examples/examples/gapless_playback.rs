/*============================================================================*/
//! Gapless Playback Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to schedule channel playback into the future with
//! sample accuracy. Use several scheduled channels to synchronize 2 or more
//! sounds.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use {
    fmod::cstr8,
    fmod_examples::{media, sleep_ms, yeet, Buttons, Example},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Note {
    C,
    D,
    E,
}

static NOTES: [Note; 32] = [
    Note::E, // Ma-
    Note::D, // ry
    Note::C, // had
    Note::D, // a
    Note::E, // lit-
    Note::E, // tle
    Note::E, // lamb,
    Note::E, // .....
    Note::D, // lit-
    Note::D, // tle
    Note::D, // lamb,
    Note::D, // .....
    Note::E, // lit-
    Note::E, // tle
    Note::E, // lamb,
    Note::E, // .....
    Note::E, // Ma-
    Note::D, // ry
    Note::C, // had
    Note::D, // a
    Note::E, // lit-
    Note::E, // tle
    Note::E, // lamb,
    Note::E, // its
    Note::D, // fleece
    Note::D, // was
    Note::E, // white
    Note::D, // as
    Note::C, // snow.
    Note::C, // .....
    Note::C, // .....
    Note::C, // .....
];

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;
        system.init(100, fmod::InitFlags::Normal)?;

        // Get information needed later for scheduling. The mixer block size, and the output rate of the mixer.
        let (dsp_block_len, _) = system.get_dsp_buffer_size()?;
        let fmod::SoftwareFormat { sample_rate, .. } = system.get_software_format()?;

        // Load 3 sounds - these are just sine wave tones at different frequencies. C, D and E on the musical scale.
        let note_c = system.create_sound(media!("c.ogg"), fmod::Mode::Default)?;
        let note_d = system.create_sound(media!("d.ogg"), fmod::Mode::Default)?;
        let note_e = system.create_sound(media!("e.ogg"), fmod::Mode::Default)?;

        // Create a channelgroup that the channels will play on. We can use this channelgroup as our clock reference.
        // It also means we can pause and pitch bend the channelgroup, without affecting the offsets of the delays, because the channelgroup clock
        // which the channels feed off, will be pausing and speeding up/slowing down and still keeping the children in sync.
        let channel_group = system.create_channel_group(cstr8!("Parent"))?;

        // Play all the sounds at once! Space them apart with set delay though so that they sound like they play in order.
        let mut clock_start = 0;
        for note in NOTES {
            // Pick a note from our tune.
            let s = match note {
                Note::C => &*note_c,
                Note::D => &*note_d,
                Note::E => &*note_e,
            };

            // Play the sound on the channelgroup we want to use as the parent clock reference (for set_delay further down)
            let channel = system.create_sound_channel(s, Some(&channel_group))?;

            if clock_start == 0 {
                clock_start = channel.get_parent_dsp_clock()?;
                // Start the sound into the future, by 2 mixer blocks worth.
                // Should be enough to avoid the mixer catching up and hitting the clock value before we've finished setting up everything.
                // Alternatively the channelgroup we're basing the clock on could be paused to stop it ticking.
                clock_start += dsp_block_len as u64 * 2;
            } else {
                // Get the length of the sound in samples.
                let slen = s.get_length(fmod::TimeUnit::Pcm)?;
                // Get the default frequency that the sound was recorded at.
                let (freq, _) = s.get_defaults()?;
                // Convert the length of the sound to 'output samples' for the output timeline.
                let slen = (slen as f32 / freq * sample_rate as f32) as u32;
                // Place the sound clock start time to this value after the last one.
                clock_start += slen as u64;
            }

            // Schedule the channel to start in the future at the newly calculated channelgroup clock value.
            channel.set_delay(clock_start.., fmod::StopAction::Pause)?;
            // Unpause the sound. Note that you won't hear the sounds, they are scheduled into the future.
            channel.set_paused(false)?;
        }

        // Main loop.
        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            // Pausing the channelgroup as the clock parent, will pause any scheduled sounds from continuing
            // If you paused the channel, this would not stop the clock it is delayed against from ticking,
            // and you'd have to recalculate the delay for the channel into the future again before it was unpaused.
            if example.btn_press(Buttons::Action1) {
                let paused = channel_group.get_paused()?;
                channel_group.set_paused(!paused)?;
            }

            if example.btn_press(Buttons::Action2) {
                for _ in 0..50 {
                    let mut pitch = channel_group.get_pitch()?;
                    pitch += 0.01;
                    channel_group.set_pitch(pitch)?;
                    system.update()?;
                    sleep_ms(10);
                }
            }

            if example.btn_press(Buttons::Action3) {
                for _ in 0..50 {
                    let mut pitch = channel_group.get_pitch()?;
                    if pitch >= 0.1 {
                        pitch -= 0.01;
                    }
                    channel_group.set_pitch(pitch)?;
                    system.update()?;
                    sleep_ms(10);
                }
            }

            system.update()?;

            // Print some information
            let playing = match channel_group.is_playing() {
                Ok(x) => x,
                Err(fmod::Error::InvalidHandle) => false,
                Err(e) => yeet!(e),
            };
            let paused = match channel_group.get_paused() {
                Ok(x) => x,
                Err(fmod::Error::InvalidHandle) => false,
                Err(e) => yeet!(e),
            };
            let chans_playing = system.get_channels_playing()?.all;

            example.draw("==================================================");
            example.draw("Gapless Playback example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");
            example.draw(format_args!(
                "Press {} to toggle pause",
                Buttons::Action1.name(),
            ));
            example.draw(format_args!(
                "Press {} to increase pitch",
                Buttons::Action2.name(),
            ));
            example.draw(format_args!(
                "Press {} to decrease pitch",
                Buttons::Action3.name(),
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!(
                "Channels Playing {chans_playing} : {}",
                if paused {
                    "Paused"
                } else if playing {
                    "Playing"
                } else {
                    "Stopped"
                }
            ));

            sleep_ms(50);
        }

        // Shut down
        note_c.release()?;
        note_d.release()?;
        note_e.release()?;
        channel_group.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
