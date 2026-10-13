use build_rs::{input, output};
use std::{env, path::PathBuf};

mod link_name;
#[cfg(target_os = "linux")]
mod linux;
mod transpile;
#[cfg(windows)]
mod windows;

pub use transpile::{make_stub, transpile};

/// The different APIs provided as part of the FMOD SDK.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Lib {
    /// FMOD Core API
    Core,
    /// FMOD Studio API
    Studio,
    /// FMOD FSBank API
    FSBank,
}

impl Lib {
    /// The subdirectory name for the API within `$FMOD_HOME/api`.
    const fn api_subdir(self) -> &'static str {
        match self {
            Lib::Core => "core",
            Lib::Studio => "studio",
            Lib::FSBank => "fsbank",
        }
    }

    /// The library name as set in `Cargo.toml:package.links`.
    const fn links(self) -> &'static str {
        match self {
            Lib::Core => "fmod",
            Lib::Studio => "fmodstudio",
            Lib::FSBank => "fsbank",
        }
    }

    /// The library object name to pass to [`rustc_link_lib`].
    fn link_name(self) -> String {
        match self {
            Lib::Core => link_name::fmod(),
            Lib::Studio => link_name::fmodstudio(),
            Lib::FSBank => link_name::fsbank(),
        }
    }

    fn dep_api_path(self) -> Option<PathBuf> {
        input::dep_metadata(self.links(), "api").map(PathBuf::from)
    }
}

#[derive(Clone, Debug)]
struct DepEnv {
    api_inc_path: Option<PathBuf>,
    api_lib_path: Option<PathBuf>,
    lib_name: Option<String>,
}

impl DepEnv {
    fn new(which: Lib) -> Self {
        DepEnv {
            api_inc_path: input::dep_metadata(which.links(), "inc").map(PathBuf::from),
            api_lib_path: input::dep_metadata(which.links(), "lib").map(PathBuf::from),
            lib_name: input::dep_metadata(which.links(), "obj"),
        }
    }

    fn make_complete(self, lookup: Lookup) -> Lookup {
        Lookup {
            include_path: self.api_inc_path.unwrap_or(lookup.include_path),
            link_path: self.api_lib_path.unwrap_or(lookup.link_path),
            link_name: self.lib_name.unwrap_or(lookup.link_name),
        }
    }
}

/// The result of looking up an FMOD API.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct Lookup {
    /// The include path for the API headers.
    pub include_path: PathBuf,
    /// The library directory for the API binaries.
    pub link_path: PathBuf,
    /// The name of the object to be linked to.
    pub link_name: String,
}

impl Lookup {
    /// Emit the buildscript directives to link the resolved API.
    pub fn link(&self) {
        output::rustc_link_search(&self.link_path);
        output::rustc_link_lib(&self.link_name);
        output::rerun_if_changed(self.link_path.join(&self.link_name));
    }
}

/// Look up FMOD API paths.
///
/// Lookup finds the first available option from the following lists:
///
/// `lookup.include_path`:
/// - `$DEP_{LIB}_INC`
/// - `$DEP_{LIB}_API/inc`
/// - `$FMOD_HOME/api/{lib}/inc`
/// - `$DEP_FMOD_INC/../../../api/{lib}/inc` only if the `$DEP_FMOD_INC`
///   and `$DEP_FMOD_LIB` paths look like they are under a usual `$FMOD_HOME`.
/// - Windows: `{HKCU:\Software\FMOD Studio API Windows}/api/inc`
/// - Linux: `$(pkg-config lib{lib} --keep-system-cflags --cflags-only-I)`
///
/// `lookup.link_path`:
/// - `$DEP_{LIB}_LIB`
/// - `$DEP_{LIB}_API/lib/{arch}`
/// - `$FMOD_HOME/api/{lib}/lib/{arch}`
/// - `$DEP_FMOD_INC/../../../api/{lib}/lib/{arch}` only if the `$DEP_FMOD_INC`
///   and `$DEP_FMOD_LIB` paths look like they are under a usual `$FMOD_HOME`.
/// - Windows: `{HKCU:\Software\FMOD Studio API Windows}/api/lib/{arch}`
/// - Linux: `$(pkg-config lib{lib} --libs-only-l)`
///
/// `lookup.link_name`:
/// - `$DEP_{LIB}_OBJ`
/// - Linux: `$(pkg-config lib{lib} --keep-system-libs --libs-only-L)`
/// - `{lib}`
pub fn lookup(which: Lib) -> Lookup {
    try_lookup(which).unwrap_or_else(|| report_missing_fmod(which))
}

fn try_lookup(which: Lib) -> Option<Lookup> {
    let dep = DepEnv::new(which);

    if let Some(include_path) = &dep.api_inc_path
        && let Some(link_path) = &dep.api_lib_path
    {
        return Some(Lookup {
            include_path: include_path.clone(),
            link_path: link_path.clone(),
            link_name: dep.lib_name.unwrap_or(which.link_name()),
        });
    }

    if let Some(api_path) = find_fmod_api(which) {
        return Some(dep.make_complete(Lookup {
            include_path: api_path.join("inc"),
            link_path: api_path.join("lib").join(fmod_arch()),
            link_name: which.link_name(),
        }));
    }

    #[cfg(target_os = "linux")]
    if let Some(lookup) = linux::lookup(which) {
        return Some(dep.make_complete(lookup));
    }

    None
}

fn find_fmod_api(which: Lib) -> Option<PathBuf> {
    if let Some(api_path) = which.dep_api_path() {
        return Some(api_path);
    }
    let fmod_home = find_fmod_home()?;
    Some(fmod_home.join("api").join(which.api_subdir()))
}

pub fn find_fmod_home() -> Option<PathBuf> {
    output::rerun_if_env_changed("FMOD_HOME");
    if let Some(home) = env::var_os("FMOD_HOME") {
        home.to_str().expect("FMOD_HOME should be valid UTF-8");
        return Some(home.into());
    }

    if let Some(home) = Lib::Core.dep_api_path().as_deref().and_then(|p| p.parent()) {
        return Some(home.into());
    }

    let dep = DepEnv::new(Lib::Core);
    if let Some(api_inc_path) = &dep.api_inc_path
        && let Some(api_lib_path) = &dep.api_lib_path
    {
        // ex api_inc_path: FMOD_HOME/api/core/inc
        // ex api_lib_path: FMOD_HOME/api/core/lib/x86_64

        if let Some(maybe_fmod_home) =
            Some(api_inc_path).and_then(|p| p.parent()?.parent()?.parent())
            && *api_inc_path == maybe_fmod_home.join("api/core/inc")
            && *api_lib_path == maybe_fmod_home.join("api/core/lib").join(fmod_arch())
        {
            return Some(maybe_fmod_home.into());
        }
    }

    #[cfg(windows)]
    if let Some(fmod_home) = windows::find_fmod_home() {
        return Some(fmod_home);
    }

    None
}

pub fn fmod_arch() -> &'static str {
    let arch = input::cargo_cfg_target_arch();
    let vendor = input::cargo_cfg_target_vendor();
    let os = input::cargo_cfg_target_os();
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
        _ => panic!("unknown/unsupported FMOD platform {}", input::target()),
    }
}

fn report_missing_fmod<T>(which: Lib) -> T {
    let lib = which.api_subdir();
    #[allow(non_snake_case)]
    let LIB = which.links().to_uppercase();
    panic!(
        r#"
Failed to locate FMOD Engine installation. <https://www.fmod.com/download#fmodengine>

To specify the location of FMOD Engine, you can:
- set $FMOD_HOME to the root directory of the extracted FMOD Engine package installation
- set $DEP_{LIB}_API to the $FMOD_HOME/api/${lib} directory of the FMOD Engine package
- set $DEP_{LIB}_INC and $DEP_{LIB}_LIB to the inc and lib directories, respectively{}
"#,
        if cfg!(unix) {
            "\n- provide pkg-config description files for the fmod libraries"
        } else {
            ""
        }
    );
}

#[deprecated(note = "use [`lookup`] instead")]
pub fn fmod_core_path() -> (String, String) {
    let Lookup {
        include_path,
        link_path,
        ..
    } = lookup(Lib::Core);
    (
        include_path.into_string().unwrap(),
        link_path.into_string().unwrap(),
    )
}

#[deprecated(note = "use [`lookup`] instead")]
pub fn fmod_studio_path() -> (String, String) {
    let Lookup {
        include_path,
        link_path,
        ..
    } = lookup(Lib::Studio);
    (
        include_path.into_string().unwrap(),
        link_path.into_string().unwrap(),
    )
}

#[deprecated(note = "use [`lookup`] instead")]
pub fn fsbank_path() -> (String, String) {
    let Lookup {
        include_path,
        link_path,
        ..
    } = lookup(Lib::FSBank);
    (
        include_path.into_string().unwrap(),
        link_path.into_string().unwrap(),
    )
}
