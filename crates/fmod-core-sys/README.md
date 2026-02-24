Raw bindings to [FMOD Core](https://fmod.com/core). These are the raw
bindings — you probably want [FMOD.rs](https://lib.rs/fmod-rs) instead.

## Versioning

The bindings crate has its version derived directly from the FMOD Engine
version, for easier pinning of a specific header version. To be specific, for
a given FMOD version `aaaa.bb.cc` (`aaaa` = product version, `bb` = major
version, `cc` = minor version), the bindings crate is released as version
`bb.cc.dd+aaaa.bb.cc-BUILD`, where `BUILD` is the specific FMOD build version,
and `dd` is an FMOD.rs-specific patch number, to allow for bindings updates if
necessary, though these are expected to be quite rare in practice.

In application use, it is recommended to pin a specific version of this crate.
FMOD checks that the header version matches the dynamic library version, so if
an unsupported runtime version mismatch occurs, FMOD should fail to initialize.

This version's vendored headers are for FMOD Engine 2.03.05 (build 148280). If
you need a version which isn't published, open a GitHub issue requesting it.

## Linking

Unless overridden, this crate will link to the logging build of FMOD when using
a development profile, and the non-logging build for release profiles. This is
based on whether the used profile inherits from the `dev` or `release` profile.

If you would like to change this behavior, or are building from a host platform
where the FMOD Engine cannot be automatically located, you can specify a [build
script override][build-override] in a `config.toml`, such as:

[build-override]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts

```toml
[target.x86_64-pc-windows-msvc.fmod]
# linker configuration
rustc-link-lib = ["fmod_vc"]
rustc-link-search = ["C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/core/lib/x64"]
# library metadata
api = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api"
inc = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/core/inc"
lib = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/core/lib/x64"
version = "2.03.05"
```

The listed `version` must match this crate's vendored header version in order
for everything to work as intended. A mismatch may result in arbitrary issues,
up to and including unsoundness and undefined behavior.

### Windows Note

The Windows `.lib` files are import libraries which require the corresponding
DLL to be present at runtime. You are required to handle this requirement, as
Cargo does not add build-time search paths to the dynamic library search path.

During development, this will probably look like having the `.dll`s in the
folder that you invoke `cargo` from.
