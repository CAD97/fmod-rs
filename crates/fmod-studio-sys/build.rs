use std::env;

fn main() {
    build::rerun_if_changed("build.rs");

    link_lib();
    #[cfg(feature = "link-search")]
    link_search();
}

fn link_lib() {
    let dev = build::profile() == "debug";
    let windows = build::cargo_cfg_target_os() == "windows";
    let lib = match (dev, windows) {
        (true, true) => "fmodstudioL_vc",
        (true, false) => "fmodstudioL",
        (false, true) => "fmodstudio_vc",
        (false, false) => "fmodstudio",
    };

    build::rustc_link_lib(lib);
}

fn link_search() {
    if cfg!(windows) {
        link_search_windows();
    } else {
        panic!("failed to guess conventional FMOD Studio API path for this host");
    }
}

fn link_search_windows() {
    let arch = build::cargo_cfg_target_arch();
    let os = build::cargo_cfg_target_os();

    let program_files = env::var("ProgramFiles(x86)").expect("failed to get ProgramFiles(x86)");
    let (fmod_os, lib_dir) = match (&*os, &*arch) {
        ("windows", "x86_64") => ("Windows", "x64"),
        ("windows", "x86") => ("Windows", "x86"),
        ("linux", "arm") => ("Linux", "arm"),
        ("linux", "aarch64") => ("Linux", "arm64"),
        ("linux", "x86") => ("Linux", "x86"),
        ("linux", "x86_64") => ("Linux", "x86_64"),
        ("macos", _) => ("Mac", ""),
        _ => {
            panic!("failed to guess conventional FMOD Studio API path for this target");
        },
    };

    let link_dir = format!(
        "{program_files}\\FMOD SoundSystem\\FMOD Studio API {fmod_os}\\api\\core\\lib\\{lib_dir}"
    );
    build::rerun_if_changed(&*link_dir);
    build::rustc_link_search(&*link_dir);
}
