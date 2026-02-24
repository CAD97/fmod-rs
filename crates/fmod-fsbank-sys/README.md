Raw bindings to [FMOD FSBank](https://www.fmod.com/docs/2.02/api/fsbank-api.html). These are the raw
bindings — you probably want [FMOD.rs](https://lib.rs/fmod-rs) instead.

## Build Requirements

This crate uses the fmod-core-sys build configuration to find the FSBank files,
looking in the sibling directory based on the standard FMOD Engine packaging,
falling back to using the same directories if those folders don't exist. To
override this behavior, set `$DEP_FSBANK_INC` and `$DEP_FSBANK_LIB`.

## Runtime Requirements

The main FMOD deployment model is via dynamic library. You, the user of this
crate, are responsible for ensuring the dynamic library is available on the
load path at program startup.

## Linking

We expect the same filename decoration as in the FMOD Engine package. If you
have a nonstandard setup or otherwise want to directly control the object name
linked against, you can set `$DEP_FSBANK_OBJ`.

## Version Compatibility

Major FMOD version numbers 2.03 and 2.02 are both supported, although testing
of individual minor versions is sparse. Use other versions at your own risk.

## Stability Disclaimer

The Rust API exposed by this crate is directly generated from the FMOD headers.
Breaking changes in this crate's API is thus entirely dependent on whether FMOD
publishes changes which would be considered breaking in a Rust API.

Note that the FMOD API within a major FMOD version number is only guaranteed
stable from the perspective of the C API. Adding new optional fields to a type
is non-API-breaking in C, but API-breaking in Rust.

## Build Metadata

`$DEP_FSBANK_INC` and `$DEP_FSBANK_LIB` are provided to the build script of any
direct dependents.

## Licensing

The build code for this crate is licensed under [The MIT License][mit].
The generated bindings inherit the FMOD end user license agreement (EULA).

[mit]: <https://opensource.org/license/mit>
