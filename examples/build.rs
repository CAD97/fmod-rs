use build_rs::output;
use fmod_build_helper::find_fmod_home;
use std::{env, error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    output::rerun_if_changed("build.rs");

    let media_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/media/"));
    if !media_path.join(".ok").exists() {
        fs::create_dir_all(media_path)?;
        fs::write(media_path.join(".ok"), "")?;

        let fmod_home = find_fmod_home();
        let Some(fmod_home) = fmod_home else {
            return Ok(());
        };

        let copy_options = &fs_extra::dir::CopyOptions::new().skip_exist(true);
        let copy_items = move |path: &str| -> Result<(), Box<dyn Error>> {
            let items: Vec<_> = fs::read_dir(fmod_home.join(path))?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<Result<_, _>>()?;
            fs_extra::copy_items(&items, media_path, copy_options)?;
            Ok(())
        };

        copy_items("core/examples/media")?;
        copy_items("studio/examples/media")?;
    }

    Ok(())
}
