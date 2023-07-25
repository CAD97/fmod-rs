/*==============================================================================
//! Generate Tone Example
//! Copyright (c), Firelight Technologies Pty, Ltd 2004-2023.
//!
//! This example shows how to play generated tones using System::playDSP
//! instead of manually connecting and disconnecting DSP units.
//!
//! For information on using FMOD example code in your own programs, visit
//! https://www.fmod.com/legal
==============================================================================*/

use fmod_examples::{media, sleep_ms, Buttons, Example};

fn main() -> anyhow::Result<()> {
    let mut example = Example::init()?;

    {
        // Create a System object and initialize.
        let system = fmod::System::new()?;
        system.init(32, fmod::InitFlags::Normal)?;

        // Create an oscillator DSP units for the tone.
        let dsp = system.create_dsp_by_type(fmod::DspType::Oscillator)?;
        // dsp.set_parameter_float(fmod::DspOscillator::Rate, 440.0)?; // TODO

        while !example.btn_press(Buttons::Quit) {
            example.update()?;
            system.update()?;

            example.draw("==================================================");
            example.draw("Generate Tone Example.");
            example.draw("Copyright (c) Firelight Technologies 2004-2023.");
            example.draw("==================================================");
            example.draw("");

            sleep_ms(50);
        }
    }

    example.close()?;

    Ok(())
}
