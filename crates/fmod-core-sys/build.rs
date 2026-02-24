use build_rs::{input::*, output::*};

mod transpile;
use transpile::transpile;

fn main() {
    rerun_if_changed("build.rs");

    let [inc, lib] = fmod_path();

    metadata("inc", &inc);
    metadata("lib", &lib);

    rustc_link_search(&lib);
    rerun_if_changed(&lib);
    rustc_link_lib(&fmod_obj());
    link_extra();

    #[rustfmt::skip]
    {
        transpile(&inc, "fmod.h", &[]);
        transpile(&inc, "fmod_codec.h", &[
            (r"pub type FMOD_CODEC_(STATE|WAVEFORMAT).*?\n", ""), // nonopaque
            ]);
        transpile(&inc, "fmod_common.h", &[
            (r"FMOD_BUILDNUMBER: ::core::ffi::c_int", "FMOD_BUILDNUMBER: ::core::ffi::c_uint"),
            (r"pub type FMOD_ASYNCREADINFO.*?\n", ""), // nonopaque
        ]);
        transpile(&inc, "fmod_dsp.h", &[
            (r"pub type FMOD_(DSP_(STATE|BUFFER_ARRAY)|COMPLEX).*?\n", ""), // nonopaque
        ]);
        transpile(&inc, "fmod_dsp_effects.h", &[]);
        transpile(&inc, "fmod_errors.h", &[]);
        transpile(&inc, "fmod_output.h", &[
            (r"pub type FMOD_OUTPUT_(STATE|OBJECT3DINFO).*?\n", ""), // nonopaque
        ]);
    };

    if cargo_cfg_target_vendor() == "uwp" {
        transpile(&inc, "fmod_uwp.h", &[]);
    }

    if cargo_cfg_target_os() == "ios" {
        transpile(&inc, "fmod_ios.h", &[]);
    }
}

fn fmod_path() -> [String; 2] {
    match (dep_metadata("fmod", "inc"), dep_metadata("fmod", "lib")) {
        (Some(inc), Some(lib)) => [inc, lib],
        (inc_override, lib_override) => {
            let [inc, lib] = if let Some(api) = dep_metadata("fmod", "api") {
                let inc = api.clone() + "/core/inc";
                let lib = api.clone() + "/core/lib/" + fmod_arch();
                [inc, lib]
            } else {
                match &*cargo_cfg_target_vendor() {
                    "pc" => win::find_fmod_pc(),
                    "uwp" => win::find_fmod_uwp(),
                    // TODO: look for "well-known" paths on other platforms?
                    _ => None,
                }
                .unwrap_or_else(report_missing_fmod)
            };

            [inc_override.unwrap_or(inc), lib_override.unwrap_or(lib)]
        },
    }
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
        ("wasm32", _, _) => "w32",
        _ => panic!("unknown/unsupported FMOD platform {}", target()),
    }
}

fn fmod_obj() -> String {
    if let Some(obj) = dep_metadata("fmod", "obj") {
        return obj;
    }

    let vendor = cargo_cfg_target_vendor();
    let arch = cargo_cfg_target_arch();
    let profile = profile();
    let atomics = cargo_cfg_target_feature().contains(&"atomics".to_string());
    let mut obj = match (&*arch, &*profile) {
        ("wasm32", "debug") if atomics => "fmodPL",
        ("wasm32", "release") if atomics => "fmodP_reduced",
        ("wasm32", "release") => "fmod_reduced",
        (_, "debug") => "fmodL",
        (_, "release") => "fmod",
        _ => unreachable!("unexpected $PROFILE"),
    }
    .to_string();

    if vendor == "pc" && matches!(&*arch, "x86" | "x86_64") {
        obj += "_vc";
    }

    if vendor == "apple" {
        let sim = if cargo_cfg_target_abi().as_deref() == Some("sim") {
            "simulator"
        } else {
            "os"
        };
        match &*cargo_cfg_target_os() {
            "ios" => obj = obj + "_iphone" + sim,
            "tvos" => obj = obj + "_appletv" + sim,
            "visionos" => obj = obj + "_xr" + sim,
            _ => {},
        }
    }

    obj
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

#[cfg(windows)]
mod win {
    use super::*;
    use winreg::{RegKey, enums::*};

    fn from_registry(key: &str) -> Option<String> {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(key)
            .ok()?
            .get_value::<String, _>("")
            .ok()
    }

    pub fn find_fmod_pc() -> Option<[String; 2]> {
        let fmod_dir = from_registry(r"Software\FMOD Studio API Windows")?;
        Some([
            fmod_dir.clone() + "/api/core/inc",
            fmod_dir.clone() + "/api/core/lib/" + fmod_arch(),
        ])
    }

    pub fn find_fmod_uwp() -> Option<[String; 2]> {
        let fmod_dir = from_registry(r"Software\FMOD Studio API Universal Windows Platform")?;
        Some([
            fmod_dir.clone() + "/api/core/inc",
            fmod_dir.clone() + "/api/core/lib/" + fmod_arch(),
        ])
    }
}

#[cfg(not(windows))]
mod win {
    pub fn find_fmod_pc() -> Option<[String; 2]> {
        None
    }

    pub fn find_fmod_uwp() -> Option<[String; 2]> {
        None
    }
}

fn report_missing_fmod<T>() -> T {
    panic!(
        r#"
Failed to locate FMOD Engine installation. <https://www.fmod.com/download#fmodengine>
To manually specify the location of FMOD Engine, use a build script override.
<https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts>

    [target.x86_64-pc-windows-msvc.fmod]
    # linker configuration
    rustc-link-lib = ["fmod_vc"]
    rustc-link-search = ["C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/core/lib/x64"]
    # library metadata
    api = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api"
    inc = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/core/inc"
    lib = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/core/lib/x64"
    version = "2.03.05"
"#
    );
}
