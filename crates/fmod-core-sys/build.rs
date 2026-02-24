use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::metadata=version=2.02.22");

    let Some([api, inc, lib]) = find_fmod() else {
        report_missing_fmod();
    };

    let vendor = &*env::var("CARGO_CFG_TARGET_VENDOR").unwrap();
    let profile = &*env::var("PROFILE").unwrap();
    let obj = match (vendor, profile) {
        ("pc", "debug") => "fmodL_vc",
        ("pc", "release") => "fmod_vc",
        (_, "debug") => "fmodL",
        (_, "release") => "fmod",
        _ => unreachable!("unexpected $PROFILE"),
    };

    println!("cargo::metadata=api={}", api.display());
    println!("cargo::metadata=inc={}", inc.display());
    println!("cargo::metadata=lib={}", lib.display());
    println!("cargo::rustc-link-lib={}", obj);
    println!("cargo::rustc-link-search={}", lib.display());
}

fn find_fmod() -> Option<[PathBuf; 3]> {
    let vendor = &*env::var("CARGO_CFG_TARGET_VENDOR").ok()?;
    match vendor {
        "pc" => find_fmod_win(),
        "uwp" => find_fmod_uwp(),
        _ => None,
    }
}

#[cfg(windows)]
fn fmod_from_registry(key: &str) -> Option<PathBuf> {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(key)
        .ok()?
        .get_value::<String, _>("")
        .ok()
        .map(PathBuf::from)
}

#[cfg(windows)]
fn find_fmod_win() -> Option<[PathBuf; 3]> {
    let fmod = fmod_from_registry(r"Software\FMOD Studio API Windows")?;
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH").ok()? {
        "i686" => "x86",
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => return None,
    };

    Some([
        fmod.join("api"),
        fmod.join("api/core/inc"),
        fmod.join("api/core/lib").join(arch),
    ])
}

#[cfg(windows)]
fn find_fmod_uwp() -> Option<[PathBuf; 3]> {
    let fmod = fmod_from_registry(r"Software\FMOD Studio API Universal Windows Platform")?;
    let arch = match &*env::var("CARGO_CFG_TARGET_ARCH").ok()? {
        "i686" => "x86",
        "x86_64" => "x64",
        "aarch64" => "arm",
        _ => return None,
    };

    Some([
        fmod.join("api"),
        fmod.join("api/core/inc"),
        fmod.join("api/core/lib").join(arch),
    ])
}

#[cfg(not(windows))]
fn find_fmod_win() -> Option<[PathBuf; 3]> {
    None
}

#[cfg(not(windows))]
fn find_fmod_uwp() -> Option<[PathBuf; 3]> {
    None
}

fn report_missing_fmod() -> ! {
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
