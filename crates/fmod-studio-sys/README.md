Raw bindings to [FMOD Studio](https://fmod.com/studio). These are the raw
bindings — you probably want [FMOD.rs](https://lib.rs/fmod-rs) instead.

## Versioning

The bindings crate has its version derived directly from the FMOD library
version, for easier pinning of a specific header version. To be specific, for
a given FMOD version `aaaa.bb.cc` (`aaaa` = product version, `bb` = major
version, `cc` = minor version), the bindings crate is released as version
`bb.cc.dd+aaaa.bb.cc-BUILD`, where `BUILD` is the specific FMOD build version,
and `dd` is an FMOD.rs-specific patch number, to allow for bindings updates if
necessary, though these are expected to be quite rare in practice.

In application use, it is recommended to pin a specific version of this crate.
FMOD checks that the header version matches the dynamic library version, so if
a version mismatch occurs, FMOD will fail to initialize.

If you need an older header version, open an issue.

The currently vendored headers are for FMOD Engine 2.02.22 (build 142841).

## Linking

By default, this crate links to `fmodstudioL` for development builds and
`fmodstudio` for release builds. This can be overridden using the
[`[target.*.fmodstudio]`][links] `config.toml` key.

[links]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts

The `link-search` optional feature will instruct this crate to add the host's
conventional install location for the FMOD Studio API to the link search path.
If this is not known for the current host, the buildscript will panic,
requiring the use of `config.toml` to override the build script link config.

### Windows Note

The Windows `.lib` files use a `_vc` decoration. The crate expects this to be
there and appropriately links to `fmodstudio_vc`/`fmodstudioL_vc` on Windows.
