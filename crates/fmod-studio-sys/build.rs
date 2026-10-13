use build_rs::output;
use fmod_build_helper::{Lib, lookup, make_stub, transpile};

#[rustfmt::skip]
static HEADERS: &[(&str, &[(&str, &str)])] = &[
    ("fmod_studio.h", &[]),
    ("fmod_studio_common.h", &[]),
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

    let lib = lookup(Lib::Studio);
    lib.link();
    output::metadata("inc", lib.include_path.to_str().unwrap());
    output::metadata("lib", lib.link_path.to_str().unwrap());

    for (header, extra_fixup) in HEADERS {
        transpile(&lib.include_path, header, extra_fixup);
    }
}
