use build_rs::output::*;
use fmod_build_helper::fmod_core_path;
use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    rerun_if_changed("build.rs");

    let media_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/media/"));
    if !media_path.join(".ok").exists() {
        fs::create_dir_all(media_path)?;

        let fmod_core_inc_path = fmod_core_path().0;
        let fmod_home = Path::new(&fmod_core_inc_path).join("../..");
        let copy_options = &fs_extra::dir::CopyOptions::new().skip_exist(true);

        let path = fmod_home.join("core/examples/media");
        let media = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fs_extra::copy_items(&media, media_path, copy_options)?;

        let path = fmod_home.join("studio/examples/media");
        let media = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fs_extra::copy_items(&media, media_path, copy_options)?;

        fs::write(media_path.join(".ok"), "")?;
    }

    Ok(())
}
