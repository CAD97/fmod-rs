use std::{env::home_dir, path::Path};

pub fn pkg_config(pkg: &str) -> Option<(String, String)> {
    let pkg = pkg_config::Config::new()
        .range_version("2.02.00".."2.04.00")
        .cargo_metadata(false)
        .env_metadata(true)
        .probe(pkg);
    match pkg {
        Err(pkg_config::Error::EnvNoPkgConfig(_)) => None,
        Err(e) => panic!("pkg-config error: {e}"),
        Ok(pkg) => {
            assert_eq!(pkg.link_paths.len(), 1);
            assert_eq!(pkg.include_paths.len(), 1);
            let pkg_lib = pkg.link_paths[0].to_string_lossy().into_owned();
            let pkg_inc = pkg.include_paths[0].to_string_lossy().into_owned();
            Some((pkg_inc, pkg_lib))
        },
    }
}

pub fn find_fmod_linux() -> Option<[String; 2]> {
    let fmod_home = None
        .or_else(find_fmod_in_dotlocal_opt)
        .or_else(find_fmod_in_usr_local_opt)
        .or_else(find_fmod_in_root_opt)?;
    Some([
        fmod_home.clone() + "/api/core/inc",
        fmod_home.clone() + "/api/core/lib/" + crate::fmod_arch(),
    ])
}

fn find_fmod_in_root_opt() -> Option<String> {
    if_exists("/opt/fmodstudioapi")
}

fn find_fmod_in_usr_local_opt() -> Option<String> {
    if_exists("/usr/local/opt/fmodstudioapi")
}

fn find_fmod_in_dotlocal_opt() -> Option<String> {
    if_exists(home_dir()?.join(".local/opt/fmodstudioapi"))
}

fn if_exists(p: impl AsRef<Path>) -> Option<String> {
    let p = p.as_ref();
    p.exists().then_some(p.to_str()?.to_string())
}
