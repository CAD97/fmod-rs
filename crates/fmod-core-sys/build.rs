fn main() {
    let arch = build::cargo_cfg_target_arch();
    let dev = build::profile() == "debug";

    build::rustc_link_search(format!("lib/{arch}"));
    if dev {
        build::rustc_link_lib("fmodL");
    } else {
        build::rustc_link_lib("fmod");
    }
}
