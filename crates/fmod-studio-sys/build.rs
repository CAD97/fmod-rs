fn main() {
    let arch = match &*build::cargo_cfg_target_arch() {
        "x86" => "x86",
        "x86_64" => "x64",
        _ => panic!("unsupported fmod arch"),
    };
    let dev = build::profile() == "dev";

    build::rustc_link_search(format!("lib/{arch}"));
    if dev {
        build::rustc_link_lib("fmodstudioL");
    } else {
        build::rustc_link_lib("fmodstudio");
    }

    bindgen::builder()
        .header("inc/fmod_studio.h")
        .generate_inline_functions(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // blocklist types from fmod-core-sys
        .blocklist_file("inc/fmod.h")
        .blocklist_file("inc/fmod_codec.h")
        .blocklist_file("inc/fmod_common.h")
        .blocklist_file("inc/fmod_dsp.h")
        .blocklist_file("inc/fmod_dsp_effects.h")
        .blocklist_file("inc/fmod_dsp_errors.h")
        .blocklist_file("inc/fmod_dsp_output.h")
        .blocklist_file("inc/fmod_errors.h")
        .blocklist_file("inc/fmod_output.h")
        .generate()
        .expect("unable to generate bindings")
        .write_to_file(build::out_dir().join("bindings.rs"))
        .expect("unable to write bindings");
}
