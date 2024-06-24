Raw bindings to [FMOD FSBank](https://www.fmod.com/docs/2.02/api/fsbank-api.html). These are the raw
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

This version's vendored headers are for FMOD Engine 2.02.22 (build 142841). If
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
[target.x86_64-pc-windows-msvc.fmodstudio]
# linker configuration
rustc-link-lib = ["fsbank_vc"]
rustc-link-search = ["C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/fsbank/lib/x64"]
# library metadata
inc = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/fsbank/inc"
lib = "C:/Program Files (x86)/FMOD SoundSystem/FMOD Studio API Windows/api/fsbank/lib/x64"
```

Unless you are changing the `rustc-link-lib` key, you shouldn't have to do this,
as the correct paths are derived from the ones provided by `fmod-core-sys`.

### Windows Note

The Windows `.lib` files are import libraries which require the corresponding
DLL to be present at runtime. You are required to handle this requirement, as
Cargo does not add build-time search paths to the dynamic library search path.

During development, this will probably look like having the `.dll`s in the
folder that you invoke `cargo` from.
