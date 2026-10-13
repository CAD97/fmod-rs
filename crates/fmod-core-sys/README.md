Raw bindings to [FMOD Core](https://fmod.com/core). These are the raw
bindings — you probably want [FMOD.rs](https://lib.rs/fmod-rs) instead.

## Build Requirements

On Windows, the FMOD Engine installation can be found automatically using the
registry key `HKCU:\Software\FMOD Studio API Windows` set by the installer.

On Linux, `pkg-config` is queried for `libfmod` in release builds or
`libfmodL` in release builds.

On any system, you can specify where FMOD Core can be found by setting:
- `$DEP_FMOD_INC` to the include path and `$DEP_FMOD_LIB` to the link search path,
- `$DEP_FMOD_API` to the FMOD Engine `api` subdirectory, or
- `$FMOD_HOME` to the FMOD Engine directory.

## Runtime Requirements

The main FMOD deployment model is via dynamic library. You, the user of this
crate, are responsible for ensuring the dynamic library is available on the
load path at program startup.

By default, this crate links to the logging version of FMOD in debug builds and
the production version of FMOD in release builds, using the dylib name as in
the unmodified FMOD Engine. This can be overriden by setting `$DEP_FMOD_OBJ`.

## Version Compatibility

FMOD versions 2.02, 2.03, and 2.04 are all supported, although testing of
individual minor versions is sparse. Use other versions at your own risk.

## Stability Disclaimer

The Rust API exposed by this crate is directly generated from the FMOD headers.
The content of the API is thus determined by which version of FMOD you use, and
changing which version of FMOD you use will change this crate's API with it.

Note that the FMOD API within a major FMOD version number is only stable from
the perspective of the C API. For example, adding new fields to a struct can
be non-API-breaking in C, but is API-breaking in Rust.

## Build Metadata

`$DEP_FMOD_INC`, `$DEP_FMOD_LIB`, and `$DEP_FMOD_VERSION` are provided to the
build script of any direct dependents. The version is provided in the `2.04.99`
string format.

## Licensing

The generated bindings inherit FMOD's end user license agreement (EULA).
