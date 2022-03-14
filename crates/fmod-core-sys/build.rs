fn main() {
    let arch = match &*build::cargo_cfg_target_arch() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("unsupported fmod arch"),
    };
    let dev = build::profile() == "dev";

    build::rustc_link_search(format!("lib/{arch}"));
    if dev {
        build::rustc_link_lib("fmodL");
    } else {
        build::rustc_link_lib("fmod");
    }

    bindgen::builder()
        .header("inc/fmod.h")
        .generate_inline_functions(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("unable to generate bindings")
        .write_to_file(build::out_dir().join("bindings.rs"))
        .expect("unable to write bindings");
}
