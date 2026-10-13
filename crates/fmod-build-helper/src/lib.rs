use build_rs::{input::*, output::rerun_if_env_changed};
use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
mod linux;
mod transpile;
#[cfg(windows)]
mod win;

pub use transpile::transpile;

pub fn fmod_core_path() -> (String, String) {
    lookup("core").unwrap_or_else(|| report_missing_fmod("core"))
}
pub fn fmod_studio_path() -> (String, String) {
    lookup("fmodstudio").unwrap_or_else(|| report_missing_fmod("studio"))
}

pub fn fsbank_path() -> (String, String) {
    lookup("fsbank").unwrap_or_else(|| report_missing_fmod("fsbank"))
}

fn lookup(pkg: &str) -> Option<(String, String)> {
    let dep_inc = dep_metadata(pkg, "inc");
    let dep_lib = dep_metadata(pkg, "lib");
    if let (Some(inc), Some(lib)) = (&dep_inc, &dep_lib) {
        return Some((inc.clone(), lib.clone()));
    }

    if let Some(api) = dep_metadata(pkg, "api") {
        let inc = api.clone() + "/inc";
        let lib = api.clone() + "/lib/" + fmod_arch();
        return Some((dep_inc.unwrap_or(inc), dep_lib.unwrap_or(lib)));
    }

    #[cfg(target_os = "linux")]
    {
        let mut pkg = pkg.to_string();
        if profile() == "debug" {
            pkg += "L";
        }
        if let Some((inc, lib)) = linux::pkg_config(&pkg) {
            return Some((dep_inc.unwrap_or(inc), dep_lib.unwrap_or(lib)));
        }
    }

    if let Some(fmod_home) = find_fmod_home() {
        return Some((
            dep_inc.unwrap_or_else(|| {
                fmod_home
                    .join("api/core/inc")
                    .to_string_lossy()
                    .into_owned()
            }),
            dep_lib.unwrap_or_else(|| {
                fmod_home
                    .join("api/core/lib")
                    .join(fmod_arch())
                    .to_string_lossy()
                    .into_owned()
            }),
        ));
    }

    None
}

fn find_fmod_home() -> Option<PathBuf> {
    rerun_if_env_changed("FMOD_HOME");
    if let Some(home) = env::var_os("FMOD_HOME") {
        return Some(home.into());
    }

    if let Some(fmod_inc) = dep_metadata("fmod", "inc")
        && let Some(fmod_lib) = dep_metadata("fmod", "lib")
    {
        let fmod_inc = Path::new(&fmod_inc);
        let fmod_lib = Path::new(&fmod_lib);

        // ex fmod_inc: FMOD_HOME/api/core/inc
        // ex fmod_lib: FMOD_HOME/api/core/lib/x86_64

        if let Some(maybe_fmod_home) = Some(fmod_inc).and_then(|p| p.parent()?.parent()?.parent())
            && fmod_inc == maybe_fmod_home.join("api/core/inc")
            && fmod_lib == maybe_fmod_home.join("api/core/lib").join(fmod_arch())
        {
            return Some(maybe_fmod_home.into());
        }
    }

    #[cfg(windows)]
    {
        if let Some(fmod_home) = win::find_fmod_home() {
            return Some(fmod_home);
        }
    }

    None
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

To specify the location of FMOD Engine, you can:
- set $FMOD_HOME to the root directory of the FMOD Engine package installation
- set $DEP_{LIB}_API to the $FMOD_HOME/api/${lib} directory of the FMOD Engine package
- set $DEP_{LIB}_INC and $DEP_{LIB}_LIB to the inc and lib directories, respectively{}
"#,
        if cfg!(unix) {
            "\n- install pkg-config description files for the fmod libraries"
        } else {
            ""
        }
    );
}
