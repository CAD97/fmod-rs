use build_rs::{input::*, output::*};
use std::{fs, path::Path};

mod transpile;
use transpile::transpile;

fn main() {
    rerun_if_changed("build.rs");

    let [inc, lib] = fsbank_path();

    metadata("inc", &inc);
    metadata("lib", &lib);

    rustc_link_search(&lib);
    rerun_if_changed(&lib);
    rustc_link_lib(&fsbank_obj());

    transpile(&inc, "fsbank.h", &[]);
    transpile(&inc, "fsbank_errors.h", &[]);
}

fn fsbank_path() -> [String; 2] {
    let inc = dep_metadata("fsbank", "inc").unwrap_or_else(|| {
        let fmod_inc = dep_metadata("fmod", "inc").unwrap();
        let inc = Path::new(&fmod_inc).join("../../fsbank/inc");
        if fs::exists(&inc).unwrap_or_default() {
            inc.to_str().unwrap().to_string()
        } else {
            fmod_inc
        }
    });

    let lib = dep_metadata("fsbank", "lib").unwrap_or_else(|| {
        let fmod_lib = dep_metadata("fmod", "lib").unwrap();
        let expected_sibling = if cargo_cfg_target_vendor() == "apple" {
            "../../fsbank/lib/"
        } else {
            "../../../fsbank/lib"
        };
        let lib = Path::new(&fmod_lib)
            .join(expected_sibling)
            .join(fmod_arch());
        if fs::exists(&lib).unwrap_or_default() {
            lib.to_str().unwrap().to_string()
        } else {
            fmod_lib
        }
    });

    [inc, lib]
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

fn fsbank_obj() -> String {
    if let Some(obj) = dep_metadata("fsbank", "obj") {
        return obj;
    }

    let vendor = cargo_cfg_target_vendor();
    let arch = cargo_cfg_target_arch();
    let mut obj = "fsbank".to_string();

    if vendor == "pc" && matches!(&*arch, "x86" | "x86_64") {
        obj += "_vc";
    }

    obj
}
