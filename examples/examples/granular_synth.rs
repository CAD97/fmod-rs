#![allow(static_mut_refs)]
// TODO: refactor to avoid static mut

/*============================================================================*/
//! Granular Synthesis Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2024.
//!
//! This example shows how you can play a string of sounds together without gaps,
//! using the setDelay command, to produce a granular synthesis style truck engine
//! effect.
//!
//! The basic operation is:
//!
//!  * Play 2 sounds initially at the same time, the first sound immediately, and
//!    the 2nd sound with a delay calculated by the length of the first sound.
//!  * Call setDelay to initiate the delayed playback. setDelay is sample accurate
//!    and uses -output- samples as the time frame, not source samples. These
//!    samples are a fixed amount per second regardless of the source sound format,
//!    for example, 48000 samples per second if FMOD is initialized to 48khz output.
//!  * Output samples are calculated from source samples with a simple
//!    source->output sample rate conversion. i.e.
//!         sound_length *= output_rate
//!         sound_length /= sound_frequency
//!  * When the first sound finishes, the second one should have automatically
//!    started. This is a good oppurtunity to queue up the next sound. Repeat
//!    step 2.
//!  * Make sure the framerate is high enough to queue up a new sound before the
//!    other one finishes otherwise you will get gaps.
//!
//! These sounds are not limited by format, channel count or bit depth like the
//! realtimestitching example is, and can also be modified to allow for overlap,
//! by reducing the delay from the first sound playing to the second by the overlap
//! amount.
//!
//!     #define USE_STREAMS = Use 2 stream instances, created while they play.
//!     #define USE_STREAMS = Use 6 static wavs, all loaded into memory.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
/*============================================================================*/

use fmod::HandleExt;
use fmod_examples::{media, sleep_ms, Buttons, Example};
use rand::prelude::*;
use std::sync::OnceLock;

macro_rules! if_streams {
    ( {$($on:tt)*} else {$($off:tt)*} ) => {
        // $($on)*
        $($off)*
    };
}

static SYSTEM: OnceLock<&'static fmod::System> = OnceLock::new();

if_streams! {{
    // Use some longer sounds, free and load them on the fly.
    // 2 streams active, double buffer them.
    static mut SOUNDS: [Option<fmod::Handle<'static, fmod::Sound>>; 2] = [const { None }; 2];
    const SOUND_NAMES: [&fmod::CStr8; 3] = [media!("c.ogg"), media!("d.ogg"), media!("e.ogg")];
} else {
    // These sounds will be loaded into memory statically.
    // 6 sounds active, one for each wav.
    static mut SOUNDS: [Option<fmod::Handle<'static, fmod::Sound>>; 6] = [const { None }; 6];
    const SOUND_NAMES: [&fmod::CStr8; 6] = [
        media!("granular/truck_idle_off_01.wav"),
        media!("granular/truck_idle_off_02.wav"),
        media!("granular/truck_idle_off_03.wav"),
        media!("granular/truck_idle_off_04.wav"),
        media!("granular/truck_idle_off_05.wav"),
        media!("granular/truck_idle_off_06.wav"),
    ];
}}

unsafe fn queue_next_sound(
    output_rate: u32,
    playing_channel: Option<&fmod::Channel>,
    new_index: usize,
    slot: usize,
) -> fmod::Result<&'static fmod::Channel> {
    let system = *SYSTEM.get().unwrap();

    let new_sound = if_streams! {{
        // Create a new stream
        unsafe {
            SOUNDS[slot] = Some(system.create_sound_ex(
                SOUND_NAMES[new_index].as_ptr(),
                fmod::Mode::CreateStream | fmod::Mode::IgnoreTags | fmod::Mode::LowMem,
                fmod::CreateSoundEx::new().suggested_sound_type(fmod::SoundType::OggVorbis),
            )?);
            SOUNDS[slot].as_deref().unwrap()
        }
    } else {{
        // Use an existing sound that was passed into us
        let _ = slot;
        SOUNDS[new_index].as_deref().unwrap()
    }}};

    let new_channel = system.create_sound_channel(new_sound, None)?;

    if let Some(playing_channel) = playing_channel {
        // Get the start time of the playing channel.
        let (delay, _) = playing_channel.get_delay()?;
        let mut start_delay = delay.start;

        // Grab the length of the playing sound, and its frequency, so we can calculate where to place the new sound on the time line.
        let playing_sound = playing_channel.get_current_sound()?.unwrap();
        let mut sound_length = playing_sound.get_length(fmod::TimeUnit::Pcm)?;
        let sound_frequency = playing_channel.get_frequency()?;

        // Now calculate the length of the sound in 'output samples'.
        // Ie if a 44khz sound is 22050 samples long, and the output rate is 48khz, then we want to delay by 24000 output samples.
        sound_length *= output_rate;
        sound_length /= sound_frequency as u32;

        // Add output rate adjusted sound length, to the clock value of the sound that is currently playing
        start_delay += sound_length as u64;

        // Set the delay of the new sound to the end of the old sound
        new_channel.set_delay(start_delay.., Default::default())?;
    } else {
        let (buffer_length, _) = system.get_dsp_buffer_size()?;

        let mut start_delay = new_channel.get_parent_dsp_clock()?;

        start_delay += 2 * buffer_length as u64;
        new_channel.set_delay(start_delay.., Default::default())?;
    }

    {
        let mut val = new_channel.get_frequency()?;
        let variation = thread_rng().gen_range(-1.0..1.0f32);
        val *= 1.0 + variation * 0.02; // @22khz, range fluctuates from 21509 to 22491
        new_channel.set_frequency(val)?;

        let mut val = new_channel.get_volume()?;
        let variation = thread_rng().gen_range(0.0..1.0f32);
        val *= 1.0 - variation * 0.2; // 0.8 to 1.0
        new_channel.set_volume(val)?;
    }

    new_channel.set_paused(false)?;

    Ok(new_channel)
}

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        SYSTEM
            .set(fmod::Handle::leak(fmod::System::new()?))
            .unwrap();
        let system = *SYSTEM.get().unwrap();
        system.init(100, fmod::InitFlags::Normal)?;
        let output_rate = system.get_software_format()?.sample_rate as u32;

        if_streams! {{} else {
            for i in 0..SOUND_NAMES.len() {
                unsafe {
                    SOUNDS[i] = Some(system.create_sound(SOUND_NAMES[i], fmod::Mode::IgnoreTags)?);
                }
            }
        }}

        // Kick off the first 2 sounds.  First one is immediate, second one will be triggered to start after the first one.
        let mut channels = unsafe {
            let channel0 = queue_next_sound(
                output_rate,
                None,
                thread_rng().gen_range(0..SOUND_NAMES.len()),
                0,
            )?;
            let channel1 = queue_next_sound(
                output_rate,
                Some(channel0),
                thread_rng().gen_range(0..SOUND_NAMES.len()),
                1,
            )?;
            [channel0, channel1]
        };

        let mut slot = 0;
        let mut paused = false;
        while !example.btn_down(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::Action1) {
                paused = !paused;

                let master_group = system.get_master_channel_group()?;
                master_group.set_paused(paused)?;
            }

            system.update()?;

            // Replace the sound that just finished with a new sound, to create endless seamless stitching!
            let is_playing = match channels[slot].is_playing() {
                Err(fmod::Error::InvalidHandle) => false,
                result => result?,
            };

            if !is_playing && !paused {
                unsafe {
                    if_streams! {{
                        // Release the sound that isn't playing any more.
                        SOUNDS[slot].release()?;
                    } else {}}

                    // Replace sound that just ended with a new sound, queued up to trigger exactly after the other sound ends.
                    channels[slot] = queue_next_sound(
                        output_rate,
                        Some(channels[1 - slot]),
                        thread_rng().gen_range(0..SOUND_NAMES.len()),
                        slot,
                    )?;
                    slot = 1 - slot; // flip
                }
            }

            example.draw("==================================================");
            example.draw("Granular Synthesis SetDelay Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2024.");
            example.draw("==================================================");
            example.draw("");
            example.draw(
                "Toggle if_streams! on/off in code to switch between streams and static samples.",
            );
            example.draw("");
            example.draw(format_args!("Press {} to pause", Buttons::Action1.name()));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");
            example.draw(format_args!(
                "Channels are {}",
                if paused { "paused" } else { "playing" }
            ));

            // If you wait too long, ie longer than the length of the shortest sound, you will get gaps.
            sleep_ms(10);
        }

        // Shut down
        unsafe {
            for sound in &mut SOUNDS {
                sound.release()?;
            }

            fmod::Handle::unleak(system).release()?;
        }
    }

    example.close()?;

    Ok(())
}
