Raw bindings to [FMOD Core](https://fmod.com/core). These are the raw
bindings — you probably want [FMOD.rs](https://lib.rs/fmod-rs) instead.

## Build Requirements

On a Windows machine building for Windows, the FMOD Engine installation can be
autodetected. On other platforms, you will need to set `$DEP_FMOD_API` to
`$FMOD_PATH/api/core`, where `$FMOD_PATH` is the path that you extracted the
FMOD Engine package. Or, if you have a custom setup that doesn't keep the FMOD
Engine's default file layout, you can set `$DEP_FMOD_INC` to the folder where
the FMOD headers are and `$DEP_FMOD_LIB` to the folder containing the FMOD
dynamic libraries.

## Runtime Requirements

The main FMOD deployment model is via dynamic library. You, the user of this
crate, are responsible for ensuring the dynamic library is available on the
load path at program startup.

## Logging

If this crate is built with a debug profile, we link against the logging build
of FMOD. If it is built with a release profile, we link against the production
non-logging binary.

We expect the same filename decoration as in the FMOD Engine package. If you
have a nonstandard setup or otherwise want to directly control the object name
linked against, you can set `$DEP_FMOD_OBJ`.

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

`$DEP_FMOD_INC`, `$DEP_FMOD_LIB`, and `$DEP_FMOD_VERSION` are provided to the
build script of any direct dependents. The version is provided in the `2.03.99`
string format.

## Licensing

The build code for this crate is licensed under [The MIT License][mit].
The generated bindings inherit the FMOD end user license agreement (EULA).

[mit]: <https://opensource.org/license/mit>
