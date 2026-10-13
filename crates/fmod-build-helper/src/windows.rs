use std::path::PathBuf;
use winreg::{RegKey, enums::*};

fn path_from_registry(key: &str) -> Option<PathBuf> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(key)
        .ok()?
        .get_value::<String, _>("")
        .ok()
        .map(PathBuf::from)
}

pub fn find_fmod_pc() -> Option<[PathBuf; 2]> {
    let fmod_dir = path_from_registry(r"Software\FMOD Studio API Windows")?;
    Some([
        fmod_dir.join("api/core/inc"),
        fmod_dir.join("api/core/lib").join(crate::fmod_arch()),
    ])
}

pub fn find_fmod_uwp() -> Option<[PathBuf; 2]> {
    let fmod_dir = path_from_registry(r"Software\FMOD Studio API Universal Windows Platform")?;
    Some([
        fmod_dir.join("api/core/inc"),
        fmod_dir.join("api/core/lib").join(crate::fmod_arch()),
    ])
}
