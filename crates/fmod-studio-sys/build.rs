fn main() {
    let arch = match &*build::cargo_cfg_target_arch() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("unsupported fmod arch"),
    };
    let dev = build::profile() == "debug";

    build::rustc_link_search(format!("lib/{arch}"));
    if dev {
        build::rustc_link_lib("fmodstudioL");
    } else {
        build::rustc_link_lib("fmodstudio");
    }
}
