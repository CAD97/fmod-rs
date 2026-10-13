use build_rs::input;

pub fn fmod() -> String {
    let wasm32 = input::cargo_cfg_target_arch() == "wasm32";
    let atomics = input::cargo_cfg_target_feature().contains(&"atomics".to_string());
    let obj = match &*input::profile() {
        "debug" if wasm32 && atomics => "fmodPL",
        "release" if wasm32 && atomics => "fmodP_reduced",
        "release" if wasm32 => "fmod_reduced",
        "debug" => "fmodL",
        "release" => "fmod",
        _ => unreachable!("unexpected $PROFILE"),
    };
    decorate(obj)
}

pub fn fmodstudio() -> String {
    let wasm32 = input::cargo_cfg_target_arch() == "wasm32";
    let atomics = input::cargo_cfg_target_feature().contains(&"atomics".to_string());
    let obj = match &*input::profile() {
        "debug" if wasm32 && atomics => "fmodstudioPL",
        "release" if wasm32 && atomics => "fmodstudioP",
        "debug" => "fmodstudioL",
        "release" => "fmodstudio",
        _ => unreachable!("unexpected $PROFILE"),
    };
    decorate(obj)
}

pub fn fsbank() -> String {
    decorate("fsbank")
}

fn decorate(obj: &str) -> String {
    let mut obj = obj.to_string();

    let vendor = input::cargo_cfg_target_vendor();
    let arch = input::cargo_cfg_target_arch();
    let abi = input::cargo_cfg_target_abi();
    let os = input::cargo_cfg_target_os();

    if vendor == "pc" && matches!(&*arch, "x86" | "x86_64") {
        obj += "_vc";
    }

    if vendor == "apple" {
        let sim = if abi.as_deref() == Some("sim") {
            "simulator"
        } else {
            "os"
        };
        match &*os {
            "ios" => obj = obj + "_iphone" + sim,
            "tvos" => obj = obj + "_appletv" + sim,
            "visionos" => obj = obj + "_xr" + sim,
            _ => {},
        }
    }

    obj
}
