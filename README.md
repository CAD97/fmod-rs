# FMOD.rs

Bindings to the [FMOD adaptive audio solution](https://fmod.com/).

![What is Adaptive Audio?](https://youtu.be/p-FLWabby4Y)

## Licensing

This is up top because it's important: FMOD is _not_ free software! In order to
use FMOD, you must [acquire a license](https://www.fmod.com/sales). However, if
you make less than $200k revenue/year on a small (<$500k) development budget,
FMOD provides a [Free Indie License](https://www.fmod.com/sales#indie-note).

The Rust code in this repository is licensed under MIT OR Apache-2.0, but this
does not remove the requirement to comply with the FMOD license.

The examples in the [examples] folder are direct ports from the distributed
FMOD examples, and are thus under the FMOD license. Similarly, the [media]
folder is also under the FMOD license and not fit for redistribution.

## Providing the FMOD runtime

In order to better comply with the FMOD license, we do _not_ redistribute the
FMOD Engine. You should [download](https://www.fmod.com/download#fmodengine)
the engine yourself. You can then copy the `lib` folder into your crate folder
or choose to [override the build script] to locate the links manually. The DLLs
are implicitly loaded from the working directory (crate root for `cargo run`),
or you can explicitly load them before calling into FMOD.

[override the build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts

Static linking requires a commercial FMOD license for source access.
FMOD.rs does not currently support staticly linking FMOD.

## Functionality

### Complete

- Raw bindings to the FMOD C API linking and running.
- Simplest `play_sound` example runs.

### Planned (Soon™)

- Safe, Rust idiomatic API wrappers.
- All examples using Rust idiomatic APIs.
- Test build and lib loading on macOS and Linux.

### Stretch Goals

- [bevy](https://bevyengine.org/) plugin.
- 99% API coverage.
- Static linking support.
