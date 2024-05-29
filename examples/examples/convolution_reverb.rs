#![allow(unused)] // has todo!s
/*============================================================================*/
//! Convolution Reverb Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2024.
//!
//! This example shows how to set up a convolution reverb DSP as a global
//! DSP unit that can be routed into by multiple seperate channels.
//!
//! Convolution reverb uses data from a real world locations called an
//! "Impulse Response" to model the reflection of audio waves back
//! to a listener.
//!
//! Impulse Response is based on "St Andrew's Church" by
//!
//!     www.openairlib.net
//!     Audiolab, University of York
//!     Damian T. Murphy
//!     http://www.openairlib.net/auralizationdb/content/st-andrews-church
//!
//! licensed under Attribution Share Alike Creative Commons license
//! http://creativecommons.org/licenses/by-sa/3.0/
//!
//!
//! Anechoic sample "Operatic Voice" by
//!
//!     www.openairlib.net
//!     http://www.openairlib.net/anechoicdb/content/operatic-voice
//!
//! licensed under Attribution Share Alike Creative Commons license
//! http://creativecommons.org/licenses/by-sa/3.0/
//!
//!
//! ### Features Demonstrated ###
//! + FMOD_DSP_CONVOLUTION_REVERB
//! + DSP::addInput
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
//!
/*============================================================================*/

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a system object and initialize
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        // Create a new channel group to hold the convolution DSP unit
        let reverb_group = system.create_channel_group(fmod::cstr8!("reverb"))?;

        // Create a new channel group to hold all the channels and process the dry path
        let main_group = system.create_channel_group(fmod::cstr8!("main"))?;

        // Create the convultion DSP unit and set it as the tail of the channel group
        let reverb_unit = system.create_dsp_by_type(fmod::DspType::ConvolutionReverb)?;
        reverb_group.push_dsp(&reverb_unit)?;
        // let reverb_unit = fmod::Handle::leak(reverb_unit);

        // Open the impulse response wav file, but use FMOD_OPENONLY as we want
        // to read the data into a separate buffer
        let ir_sound = system.create_sound(
            media!("standrews.wav"),
            fmod::Mode::Default | fmod::Mode::OpenOnly,
        )?;

        // 🦀 FMOD.rs provides a helper for reading IR sounds
        let ir_data = fmod::effect::ConvolutionReverb::ImpulseResponse::from_sound(&ir_sound)?;
        reverb_unit.set_parameter(fmod::effect::ConvolutionReverb::Ir, &*ir_data)?;

        // Don't pass any dry signal from the reverb unit, instead take the dry part
        // of the mix from the main signal path
        reverb_unit.set_parameter(fmod::effect::ConvolutionReverb::Dry, -80.0)?;

        // We can now free our copy of the IR data and release the sound object, the reverb unit
        // has created it's internal data
        drop(ir_data);
        ir_sound.release()?;

        // Load up and play a sample clip recorded in an anechoic chamber
        let sound = system.create_sound(
            media!("singing.wav"),
            fmod::Mode::D3 | fmod::Mode::LoopNormal,
        )?;
        let channel = system.create_sound_channel(&sound, Some(&main_group))?;

        // Create a send connection between the channel head and the reverb unit
        let channel_head = unsafe { channel.get_dsp_head()? };
        let reverb_connection =
            reverb_unit.add_input(channel_head, fmod::DspConnectionType::Send)?;
        let reverb_connection = unsafe { reverb_connection.as_ref() };

        channel.set_paused(false)?;

        let mut wet_volume = 1.0_f32;
        let mut dry_volume = 1.0_f32;

        while !example.btn_press(Buttons::Quit) {
            example.update()?;

            if example.btn_press(Buttons::Left) {
                wet_volume = (wet_volume - 0.05).clamp(0.0, 1.0);
            }
            if example.btn_press(Buttons::Right) {
                wet_volume = (wet_volume + 0.05).clamp(0.0, 1.0);
            }
            if example.btn_press(Buttons::Down) {
                dry_volume = (dry_volume - 0.05).clamp(0.0, 1.0);
            }
            if example.btn_press(Buttons::Up) {
                dry_volume = (dry_volume + 0.05).clamp(0.0, 1.0);
            }

            system.update()?;

            reverb_connection.set_mix(wet_volume)?;
            main_group.set_volume(dry_volume)?;

            example.draw("==================================================");
            example.draw("Convolution Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2024.");
            example.draw("==================================================");
            example.draw(format_args!(
                "Press {} and {} to change dry mix",
                Buttons::Up.name(),
                Buttons::Down.name()
            ));
            example.draw(format_args!(
                "Press {} and {} to change wet mix",
                Buttons::Left.name(),
                Buttons::Right.name()
            ));
            example.draw(format_args!(
                "wet mix [{:.2}] dry mix [{:.2}]",
                wet_volume, dry_volume
            ));
            example.draw(format_args!("Press {} to quit", Buttons::Quit.name()));
            example.draw("");

            sleep_ms(50);
        }

        // Shut down
        sound.release()?;
        main_group.release()?;
        reverb_group.remove_dsp(&reverb_unit)?;
        reverb_unit.disconnect_all()?;
        reverb_unit.release()?;
        reverb_group.release()?;
        system.release()?;
    }

    example.close()?;

    Ok(())
}
