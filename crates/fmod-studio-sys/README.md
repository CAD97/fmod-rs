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

The currently vendored headers are for FMOD Engine 2.02.14 (build 133546).

## Linking

We add `lib/{arch}` to the search path, and link `fmodstudioL` for development
builds, `fmodstudio` for release builds. The dynamic library is implicitly
loaded from the run directory, or you can load it explicitly.

### Windows Note

FMOD provides the 64 bit windows files in a `x64` folder; we use the convention
used in the linux distribution and by the Rust toolchain, and thus you need to
rename the arch folder to `x86_64`. Additionally, the `.lib` files have a `_vc`
suffix which need to be removed such that cargo/rustc can link them properly.
