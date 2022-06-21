use std::{
    array, env,
    path::{Path, PathBuf},
};

const WINDOWS_DEFAULT_API_DIR: &str =
    r#"C:\Program Files (x86)\FMOD SoundSystem\FMOD Studio API Windows\"#;

const API_DIR_ENV_VAR: &str = "FMOD_API_DIR";
const LIB_DIR_ENV_VAR: &str = "FMOD_LIB_DIR";

fn main() {
    build::rerun_if_changed("build.rs");

    let lib = find_lib()
        .canonicalize()
        .expect("lib path should canonicalize");
    build::rustc_link_search(lib.parent().unwrap());
}

fn find_lib() -> PathBuf {
    build::rerun_if_env_changed(API_DIR_ENV_VAR);
    let explicit_api_dir = env::var_os(API_DIR_ENV_VAR);
    build::rerun_if_env_changed(LIB_DIR_ENV_VAR);
    let explicit_lib_dir = env::var_os(LIB_DIR_ENV_VAR);

    let arch = &*build::cargo_cfg_target_arch();
    let archs = arch_compatibility(&arch);

    let lib_name = lib_name();

    if let Some(lib_dir) = explicit_lib_dir {
        let lib_dir = Path::new(&lib_dir);
        println!(
            "${LIB_DIR_ENV_VAR} is set; searching `{lib_dir}` for FMOD",
            lib_dir = lib_dir.display()
        );
        match find_lib_in_lib_dir(lib_dir, archs, lib_name) {
            Some(lib) => return lib,
            None => (),
        }
    }

    if let Some(api_dir) = explicit_api_dir {
        let api_dir = Path::new(&api_dir);
        println!(
            "${API_DIR_ENV_VAR} is set; searching `{api_dir}` for FMOD",
            api_dir = api_dir.display()
        );
        match find_lib_in_api_dir(api_dir, archs, lib_name) {
            Some(lib) => return lib,
            None => (),
        }
    }

    #[cfg(windows)]
    if build::cargo_cfg_windows() {
        println!("Searching for FMOD in default Windows FMOD API directory");
        match find_lib_in_api_dir(Path::new(WINDOWS_DEFAULT_API_DIR), archs, lib_name) {
            Some(lib) => return lib,
            None => (),
        }
    }

    panic!("Could not find FMOD in any of the expected locations; please set ${LIB_DIR_ENV_VAR} or ${API_DIR_ENV_VAR}");
}

fn find_lib_in_lib_dir(lib_dir: &Path, archs: &[&str], lib_name: &str) -> Option<PathBuf> {
    let fail = || {
        println!(
            "FMOD not found in `{lib_dir}`; skipping",
            lib_dir = lib_dir.display()
        );

        println!();
        None
    };

    if !lib_dir.exists() {
        println!(
            "`{lib_dir}` does not exist; skipping",
            lib_dir = lib_dir.display()
        );
        build::rerun_if_changed(lib_dir);
        return fail();
    }

    for arch in archs {
        let mut path = lib_dir.to_owned();
        path.push(arch);
        path.push(lib_with_decorations(lib_name));
        if path.exists() {
            build::rerun_if_changed(&path);
            println!("Found FMOD at `{path}`", path = path.display());
            return Some(path);
        } else {
            println!("`{path}` does not exist; skipping", path = path.display());
            build::rerun_if_changed(path.parent().unwrap());
        }
    }

    let mut path = lib_dir.to_owned();
    path.push(lib_with_decorations(lib_name));
    if path.exists() {
        println!("Found FMOD at `{path}`", path = path.display());
        return Some(path);
    } else {
        println!("`{path}` does not exist; skipping", path = path.display());
        build::rerun_if_changed(path.parent().unwrap());
    }

    fail()
}

fn find_lib_in_api_dir(api_dir: &Path, archs: &[&str], lib_name: &str) -> Option<PathBuf> {
    let fail = || {
        println!(
            "FMOD not found in `{api_dir}`; skipping",
            api_dir = api_dir.display()
        );

        println!();
        None
    };

    if !api_dir.exists() {
        println!(
            "`{api_dir}` does not exist; skipping",
            api_dir = api_dir.display()
        );
        build::rerun_if_changed(api_dir);
        return fail();
    }

    let mut path = api_dir.to_owned();

    path.push("api");
    if path.exists() {
        println!("It looks like the API dir is the root install; entering ./api");
    } else {
        path.pop();
    }

    path.push("core");
    if !path.exists() {
        println!("`{path}` does not exist; skipping", path = path.display());
        build::rerun_if_changed(path.parent().unwrap());
        return fail();
    }

    path.push("lib");
    if !path.exists() {
        println!("`{path}` does not exist; skipping", path = path.display());
        build::rerun_if_changed(path.parent().unwrap());
        return fail();
    }

    for arch in archs {
        let mut path = path.clone();
        path.push(arch);
        path.push(lib_with_decorations(lib_name));
        if path.exists() {
            build::rerun_if_changed(&path);
            println!("Found FMOD at `{path}`", path = path.display());
            return Some(path);
        } else {
            println!("`{path}` does not exist; skipping", path = path.display());
            build::rerun_if_changed(path.parent().unwrap());
        }
    }

    fail()
}

fn arch_compatibility<'a, 'b>(arch: &'a &'b str) -> &'a [&'b str] {
    match arch {
        &"x86_64" => &["x64", "x86_64"],
        arch => array::from_ref(arch),
    }
}

fn lib_name() -> &'static str {
    let windows = build::cargo_cfg_windows() && !build::cargo_cfg_unix();

    match windows {
        true => "fmod_vc",
        false => "fmod",
    }
}

fn lib_with_decorations(lib: &str) -> String {
    match &*build::cargo_cfg_target_os() {
        "windows" => format!("{}.lib", lib),
        "macos" => format!("lib{}.dylib", lib),
        "linux" => format!("lib{}.so", lib),
        os => panic!("I don't know what library decorations look like for {os}"),
    }
}
