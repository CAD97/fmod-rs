use crate::{Lib, Lookup};
use build_rs::{input, output};
use std::assert_matches;

pub fn lookup(which: Lib) -> Option<Lookup> {
    let mut pkg = ["lib", which.links()].concat();
    if input::profile() == "debug" {
        pkg += "L";
    }

    let pkg = pkg_config::Config::new()
        .range_version("2.02.00".."2.05.00")
        .cargo_metadata(false)
        .env_metadata(true)
        .probe(&pkg)
        .inspect_err(|e| output::error(&format!("{e}")))
        .ok()?;

    assert_matches!(*pkg.libs, [_]);
    assert_matches!(*pkg.link_paths, [_]);
    assert_matches!(*pkg.link_files, []);
    assert_matches!(*pkg.frameworks, []);
    assert_matches!(*pkg.framework_paths, []);
    assert_matches!(*pkg.include_paths, [_]);
    assert_matches!(*pkg.ld_args, []);
    assert_eq!(pkg.defines.len(), 0);

    Some(Lookup {
        include_path: (&pkg.include_paths[0]).into(),
        link_path: (&pkg.link_paths[0]).into(),
        link_name: (&pkg.libs[0]).into(),
    })
}
