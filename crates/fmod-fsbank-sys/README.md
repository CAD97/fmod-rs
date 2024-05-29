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

If you need an older header version, open an issue.

The currently vendored headers are for FMOD Engine 2.02.22 (build 142841).

## Linking

The `link-search` optional feature will instruct this crate to add the host's
conventional install location for the FMOD Studio API to the link search path.
If this is not known for the current host, the buildscript will panic,
requiring the use of [`[target.*.fsbank]`][links] `config.toml` key to override
the build script link config.

[links]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts

### Windows Note

The Windows `.lib` files use a `_vc` decoration. The crate expects this to be
there and appropriately links to `fsbank_vc` on Windows.
