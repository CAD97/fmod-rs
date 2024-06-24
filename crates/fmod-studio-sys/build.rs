use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let inc: PathBuf = env::var("DEP_FMOD_INC").unwrap().into();
    let lib: PathBuf = env::var("DEP_FMOD_LIB").unwrap().into();
    let remap = |p| match p == "core" {
        true => PathBuf::from("studio"),
        false => PathBuf::from(p),
    };

    let inc: PathBuf = inc.iter().map(remap).collect();
    let lib: PathBuf = lib.iter().map(remap).collect();

    let vendor = &*env::var("CARGO_CFG_TARGET_VENDOR").unwrap();
    let profile = &*env::var("PROFILE").unwrap();
    let obj = match (vendor, profile) {
        ("pc", "debug") => "fmodstudioL_vc",
        ("pc", "release") => "fmodstudio_vc",
        (_, "debug") => "fmodstudioL",
        (_, "release") => "fmodstudio",
        _ => unreachable!("unexpected $PROFILE"),
    };

    println!("cargo::metadata=inc={}", inc.display());
    println!("cargo::metadata=lib={}", lib.display());
    println!("cargo::rustc-link-lib={}", obj);
    println!("cargo::rustc-link-search={}", lib.display());
}
