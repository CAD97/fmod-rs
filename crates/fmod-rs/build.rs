use std::env;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let Ok(fmod_version) = env::var("DEP_FMOD_VERSION") else {
        return;
    };
    let version = fmod_version.split('.').collect::<Vec<_>>();
    assert!(version.len() == 3);

    let major_version: u8 = version[1].parse().unwrap();
    let minor_version: u8 = version[2].parse().unwrap();

    println!("cargo::rustc-cfg=fmod_version_major=\"{major_version}\"");
    for version in 0..=major_version {
        println!("cargo::rustc-cfg=has_fmod_version_major=\"{version}\"");
    }

    println!("cargo::rustc-cfg=fmod_version_minor=\"{minor_version}\"");
    for version in 0..=minor_version {
        println!("cargo::rustc-cfg=has_fmod_version_minor=\"{version}\"");
    }

    println!("cargo::rustc-env=FMOD_VERSION={fmod_version}");
}
