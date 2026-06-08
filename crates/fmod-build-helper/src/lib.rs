use build_rs::input::*;
use std::{fs, path::Path};

mod transpile;
#[cfg(windows)]
mod win;

pub use transpile::transpile;

fn fmod_core_path_for(lib: &str) -> (String, String) {
    match (dep_metadata("fmod", "inc"), dep_metadata("fmod", "lib")) {
        (Some(inc), Some(lib)) => (inc, lib),
        (inc_override, lib_override) => {
            let [inc, lib] = if let Some(api) = dep_metadata("fmod", "api") {
                let inc = api.clone() + "/inc";
                let lib = api.clone() + "/lib/" + fmod_arch();
                [inc, lib]
            } else {
                match &*cargo_cfg_target_vendor() {
                    #[cfg(windows)]
                    "pc" => win::find_fmod_pc(),
                    #[cfg(windows)]
                    "uwp" => win::find_fmod_uwp(),
                    // TODO: look for "well-known" paths on other platforms?
                    _ => None,
                }
                .unwrap_or_else(|| report_missing_fmod(lib))
            };
            (inc_override.unwrap_or(inc), lib_override.unwrap_or(lib))
        },
    }
}

pub fn fmod_core_path() -> (String, String) {
    fmod_core_path_for("core")
}

pub fn fsbank_path() -> (String, String) {
    let inc = dep_metadata("fsbank", "inc").unwrap_or_else(|| {
        let fmod_inc = fmod_core_path_for("fsbank").0;
        let inc = Path::new(&fmod_inc).join("../../fsbank/inc");
        if fs::exists(&inc).unwrap_or_default() {
            inc.to_str().unwrap().to_string()
        } else {
            fmod_inc
        }
    });

    let lib = dep_metadata("fsbank", "lib").unwrap_or_else(|| {
        let fmod_lib = fmod_core_path_for("fsbank").1;
        let expected_sibling = if cargo_cfg_target_vendor() == "apple" {
            "../../fsbank/lib"
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

    (inc, lib)
}

pub fn fmod_studio_path() -> (String, String) {
    let inc = dep_metadata("fmodstudio", "inc").unwrap_or_else(|| {
        let fmod_inc = fmod_core_path_for("studio").0;
        let inc = Path::new(&fmod_inc).join("../../studio/inc");
        if fs::exists(&inc).unwrap_or_default() {
            inc.to_str().unwrap().to_string()
        } else {
            fmod_inc
        }
    });

    let lib = dep_metadata("fmodstudio", "lib").unwrap_or_else(|| {
        let fmod_lib = fmod_core_path_for("studio").1;
        let expected_sibling = if cargo_cfg_target_vendor() == "apple" {
            "../../studio/lib/"
        } else {
            "../../../studio/lib"
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

    (inc, lib)
}

pub fn fmod_arch() -> &'static str {
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
        ("wasm32", _, _) => "w32",
        _ => panic!("unknown/unsupported FMOD platform {}", target()),
    }
}

fn report_missing_fmod<T>(lib: &str) -> T {
    #[allow(non_snake_case)]
    let LIB = match lib {
        "core" => "FMOD",
        "studio" => "FMODSTUDIO",
        "fsbank" => "FSBANK",
        _ => unreachable!(),
    };
    panic!(
        r#"
Failed to locate FMOD Engine installation. <https://www.fmod.com/download#fmodengine>

To specify the location of FMOD Engine, set the env var $DEP_FMOD_API to the
api/core directory of the FMOD Engine package, or set both $DEP_{LIB}_INC
and $DEP_{LIB}_LIB to the header include and binary directories for the
{lib} library, respectively.
"#
    );
}
