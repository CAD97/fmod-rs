fn main() {
    build::rerun_if_changed("build.rs");

    let fmod_version = build::dep("fmod", "version").unwrap();
    let version = fmod_version.split('.').collect::<Vec<_>>();
    assert!(version.len() == 3);

    let product_version: u16 = version[0].parse().unwrap();
    let major_version: u8 = version[1].parse().unwrap();
    let minor_version: u8 = version[2].parse().unwrap();

    build::rustc_cfg_value("fmod_version_product", &format!("\"{product_version}\""));
    build::rustc_cfg_value("fmod_version_major", &format!("\"{major_version}\""));
    for version in 0..=minor_version {
        build::rustc_cfg_value("fmod_version_minor", &format!("\"{version}\""));
    }

    build::rustc_env("FMOD_VERSION", &fmod_version);
}
