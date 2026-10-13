use build_rs::{input, output};
use fmod_build_helper::{Lib, lookup, make_stub, transpile};
use regex::Regex;
use std::fs;

#[rustfmt::skip]
static HEADERS: &[(&str, &[(&str, &str)])] = &[
    ("fmod.h", &[]),
    ("fmod_codec.h", &[
        (r"pub type FMOD_CODEC_(STATE|WAVEFORMAT).*?\n", ""), // nonopaque
    ]),
    ("fmod_common.h", &[
        (r"FMOD_BUILDNUMBER: ::core::ffi::c_int", "FMOD_BUILDNUMBER: ::core::ffi::c_uint"),
        (r"pub type FMOD_ASYNCREADINFO.*?\n", ""), // nonopaque
    ]),
    ("fmod_dsp.h", &[
        (r"pub type FMOD_(DSP_(STATE|BUFFER(_ARRAY)?)|COMPLEX).*?\n", ""), // nonopaque
    ]),
    ("fmod_dsp_effects.h", &[]),
    ("fmod_errors.h", &[]),
    ("fmod_output.h", &[
        (r"pub type FMOD_OUTPUT_(STATE|OBJECT3DINFO).*?\n", ""), // nonopaque
    ]),
];

fn main() {
    output::rerun_if_changed("build.rs");

    output::rerun_if_env_changed("DOCS_RS");
    if std::env::var_os("DOCS_RS").is_some() {
        for (header, _) in HEADERS {
            make_stub(header);
        }
        return;
    }

    let lib = lookup(Lib::Core);
    lib.link();
    link_extra();
    output::metadata("inc", lib.include_path.to_str().unwrap());
    output::metadata("lib", lib.link_path.to_str().unwrap());

    for (header, extra_fixup) in HEADERS {
        transpile(&lib.include_path, header, extra_fixup);
    }

    if input::cargo_cfg_target_vendor() == "uwp" {
        transpile(&lib.include_path, "fmod_uwp.h", &[]);
    }

    if input::cargo_cfg_target_os() == "ios" {
        transpile(&lib.include_path, "fmod_ios.h", &[]);
    }

    let fmod_common = fs::read_to_string(lib.include_path.join("fmod_common.h")).unwrap();
    // 0xaaaabbcc -> aaaa = product version, bb = major version, cc = minor version.
    let version_pat = Regex::new(r"#define FMOD_VERSION * 0x(\d{4})(\d{2})(\d{2})").unwrap();
    let captures = version_pat.captures(&fmod_common).unwrap();
    let version = format!("{}.{}.{}", &captures[1], &captures[2], &captures[3]);
    output::metadata("version", &version);
}

fn link_extra() {
    if input::cargo_cfg_target_vendor() == "apple"
        && matches!(&*input::cargo_cfg_target_os(), "ios" | "tvos" | "visionos")
    {
        output::rustc_link_lib_kind("framework", "AudioToolbox");
        output::rustc_link_lib_kind("framework", "CoreAudio");
    }
}
