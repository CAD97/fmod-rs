Raw bindings to [FMOD Studio](https://fmod.com/studio). These are the raw
bindings — you probably want [FMOD.rs](https://lib.rs/fmod-rs) instead.

## Build Requirements

This crate uses the fmod-core-sys build configuration to find the Studio files,
looking in the sibling directory based on the standard FMOD Engine packaging,
falling back to using the same directories if those folders don't exist. To
override this behavior, set `$DEP_FMODSTUDIO_INC` and `$DEP_FMODSTUDIO_LIB`.

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
linked against, you can set `$DEP_FMODSTUDIO_OBJ`.

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

`$DEP_FMODSTUDIO_INC` and `$DEP_FMODSTUDIO_LIB`, are provided to the build
script of any direct dependents.

## Licensing

The build code for this crate is licensed under [The MIT License][mit] OR
[the Apache license version 2.0][apache-2.0]. The generated bindings inherit
FMOD's end user license agreement (EULA).

[mit]: <https://opensource.org/license/mit>
[apache-2.0]: <https://www.apache.org/licenses/LICENSE-2.0>
