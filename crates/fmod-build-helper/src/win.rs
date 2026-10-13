use winreg::{RegKey, enums::*};

fn from_registry(key: &str) -> Option<String> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(key)
        .ok()?
        .get_value::<String, _>("")
        .ok()
}

pub fn find_fmod_pc_home() -> Option<String> {
    from_registry(r"Software\FMOD Studio API Windows")
}

pub fn find_fmod_uwp_home() -> Option<String> {
    from_registry(r"Software\FMOD Studio API Universal Windows Platform")
}

pub fn find_fmod_pc() -> Option<[String; 2]> {
    let fmod_dir = from_registry(r"Software\FMOD Studio API Windows")?;
    Some([
        fmod_dir.clone() + "/api/core/inc",
        fmod_dir.clone() + "/api/core/lib/" + crate::fmod_arch(),
    ])
}

pub fn find_fmod_uwp() -> Option<[String; 2]> {
    let fmod_dir = from_registry(r"Software\FMOD Studio API Universal Windows Platform")?;
    Some([
        fmod_dir.clone() + "/api/core/inc",
        fmod_dir.clone() + "/api/core/lib/" + crate::fmod_arch(),
    ])
}
