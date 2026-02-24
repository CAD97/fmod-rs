use build_rs::{input::*, output::*};
use std::path::PathBuf;

mod transpile;
use transpile::transpile;

fn main() {
    rerun_if_changed("build.rs");

    let api = PathBuf::from(dep_metadata("fmod", "api").unwrap());
    let inc = PathBuf::from(dep_metadata("fmod", "inc").unwrap());
    let lib = PathBuf::from(dep_metadata("fmod", "lib").unwrap());

    let remap = |p| match p == "core" {
        true => PathBuf::from("studio"),
        false => PathBuf::from(p),
    };

    let inc = if inc.starts_with("api") {
        inc.iter().map(remap).collect()
    } else {
        api.join("studio/inc")
    };
    let lib = if lib.starts_with("api") {
        lib.iter().map(remap).collect()
    } else {
        let arch = fmod_arch();
        api.join("studio/lib").join(arch)
    };

    let inc: PathBuf = inc.iter().map(remap).collect();
    let lib: PathBuf = lib.iter().map(remap).collect();

    metadata("api", api.as_os_str().to_str().unwrap());
    metadata("inc", inc.as_os_str().to_str().unwrap());
    metadata("lib", lib.as_os_str().to_str().unwrap());

    rustc_link_search(&lib);
    rerun_if_changed(&lib);
    rustc_link_lib(&fmod_lib());

    link_extra();

    transpile(&inc, "fmod_studio.h", &[]);
    transpile(&inc, "fmod_studio_common.h", &[]);
}

fn fmod_arch() -> &'static str {
    let arch = cargo_cfg_target_arch();
    let vendor = cargo_cfg_target_vendor();
    let os = cargo_cfg_target_os();
    match (&*arch, &*vendor, &*os) {
        ("i686", "pc", "windows") => "x86",
        ("x86_64", "pc", "windows") => "x64",
        ("aarch64", "pc", "windows") => "arm64",
        ("i686", "uwp", "windows") => "x86",
        ("x86_64", "uwp", "windows") => "x64",
        ("aarch64", "uwp", "windows") => "arm",
        (_, "apple", _) => ".",
        ("i686", _, "linux") => "x86",
        ("x86_64", _, "linux") => "x86_64",
        ("armv7", _, "linux") => "arm",
        ("aarch64", _, "linux") => "arm64",
        ("armv7", _, "android") => "armeabi-v7a",
        ("aarch64", _, "android") => "arm64-v8a",
        ("i686", _, "android") => "x86",
        ("x86_64", _, "android") => "x86_64",
        ("wasm32", _, "emscripten") => "w32",
        _ => panic!("unknown/unsupported FMOD platform {}", target()),
    }
}

fn fmod_lib() -> String {
    _ = fmod_arch(); // ensure valid platform
    let vendor = cargo_cfg_target_vendor();
    let arch = cargo_cfg_target_arch();
    let profile = profile();
    let atomics = cargo_cfg_target_feature().contains(&"atomics".to_string());
    let mut dylib = match (&*arch, &*vendor, &*profile) {
        ("x86_64", "pc", "debug") => "fmodstudioL_vc",
        ("x86_64", "pc", "release") => "fmodstudio_vc",
        ("wasm32", _, "debug") if atomics => "fmodstudioPL",
        ("wasm32", _, "release") if atomics => "fmodstudioP",
        (_, _, "debug") => "fmodstudioL",
        (_, _, "release") => "fmodstudio",
        _ => unreachable!("unexpected $PROFILE"),
    }
    .to_string();
    if vendor == "apple" {
        let sim = if cargo_cfg_target_abi().as_deref() == Some("sim") {
            "simulator"
        } else {
            "os"
        };
        match &*cargo_cfg_target_os() {
            "ios" => dylib = dylib + "_iphone" + sim,
            "tvos" => dylib = dylib + "_appletv" + sim,
            "visionos" => dylib = dylib + "_xr" + sim,
            _ => {},
        }
    }
    dylib
}

fn link_extra() {
    if cargo_cfg_target_vendor() == "apple" {
        match &*cargo_cfg_target_os() {
            "ios" | "tvos" | "visionos" => {
                rustc_link_lib_kind("framework", "AudioToolbox");
                rustc_link_lib_kind("framework", "CoreAudio");
            },
            _ => {},
        }
    }
}
