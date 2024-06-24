use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");

    let media_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/media/"));
    if !media_path.exists() {
        fs::create_dir_all(media_path)?;
        let api = PathBuf::from(env::var("DEP_FMOD_API")?);

        let path = api.join("core/examples/media");
        let media = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fs_extra::copy_items(&media, media_path, &fs_extra::dir::CopyOptions::new())?;

        let path = api.join("studio/examples/media");
        let media = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        fs_extra::copy_items(&media, media_path, &fs_extra::dir::CopyOptions::new())?;
    }

    Ok(())
}
