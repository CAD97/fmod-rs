fn main() {
    let arch = match &*build::cargo_cfg_target_arch() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("unsupported fmod arch"),
    };
    let dev = dbg!(build::profile()) == "debug";

    build::rustc_link_search(format!("lib/{arch}"));
    if dev {
        build::rustc_link_lib("fmodL");
    } else {
        build::rustc_link_lib("fmod");
    }
}
