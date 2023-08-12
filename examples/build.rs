use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    build::rerun_if_changed("build.rs");

    let media_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/media/"));
    if !media_path.exists() {
        fs::create_dir_all(media_path)?;
        let Some(core_media_path) = core_examples_media() else {
            return Ok(());
        };
        let core_media = fs::read_dir(core_media_path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fs_extra::copy_items(&core_media, media_path, &fs_extra::dir::CopyOptions::new())?;
    }

    Ok(())
}

fn core_examples_media() -> Option<String> {
    if cfg!(windows) {
        Some(core_examples_media_windows())
    } else {
        build::warning("It looks like you don't have the examples' required media files. \
            Please download them from https://www.fmod.com/download and extract them to the media folder.");
        None
    }
}

fn core_examples_media_windows() -> String {
    let program_files = env::var("ProgramFiles(x86)").expect("failed to get ProgramFiles(x86)");
    format!(
        "{program_files}\\FMOD SoundSystem\\FMOD Studio API Windows\\api\\core\\examples\\media"
    )
}
