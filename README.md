# FMOD.rs

Bindings to the [FMOD adaptive audio engine](https://fmod.com/).
[What is Adaptive Audio? (video)](https://youtu.be/p-FLWabby4Y)

## Licensing

This is up top because it's important: FMOD is _not_ free software! In order to
use FMOD, you must [acquire a license](https://www.fmod.com/sales). However, if
you make less than $200k revenue/year on a small (<$500k) development budget,
FMOD provides a [Free Indie License](https://www.fmod.com/sales#indie-note).

The examples in the [examples](examples) folder are direct ports from the
distributed FMOD examples, and are thus under the FMOD license. The media files
used for the examples are not redistributable, and thus need to be acquired
by you alongside the binaries to run the FMOD Engine.

The Rust code in this repository is licensed under MIT OR Apache-2.0, but this
does not remove the requirement to comply with the FMOD license.

## Providing the FMOD Engine

In order to comply with the FMOD license, we do _not_ redistribute the FMOD
Engine. You must [download](https://www.fmod.com/download#fmodengine) the engine
yourself.

The currently vendored headers are for FMOD Engine 2.02.19 (build 137979). To
pin the headers to a specific build, pin `fmod-core-sys` and `fmod-studio-sys`.
The sys libraries are versioned based on the FMOD version for convenient version
pins; for example, FMOD version 2.02.19 is served by sys crates version 2.19.X.

By default, this crate links to `fmodL` for development builds and
`fmod` for release builds. This can be overridden using the
[`[target.*.fmod]` and `[target.*.fmodstudio]`][links] `config.toml` keys.

[links]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#overriding-build-scripts

The `link-search` optional feature will instruct FMOD.rs to add the host's
conventional install location for the FMOD Studio API to the link search path.
If this is not known for the current host, the buildscript will panic,
requiring the use of `config.toml` to override the build script link config.

### Windows Note

The Windows `.lib` files use a `_vc` decoration. The crate expects this to be
there and appropriately links to `fmod_vc`/`fmodL_vc` on Windows.


## Functionality

### Complete

- Raw bindings to the FMOD C API linking and running on Windows.
- Some examples ported to the wrapped API.
- Thread-safe API.

### Planned (Soon™)

- All FMOD Core examples ported using Rust idiomatic APIs.
- Occasional manual tests that linking/running works on macOS/Linux.

### Stretch Goals

- [bevy](https://bevyengine.org/) plugin.
- 99% API coverage.
- Thread-unsafe use[^1].
- Static linking support.
- Safety audit against misuse.
- Support multiple FMOD versions.

-----

[^1]: FMOD is thread safe by default. For specifics on FMOD thread safety, see
[Threads and Thread Safety]. For our purposes, FMOD _is_ threadsafe, _unless_
`FMOD_INIT_THREAD_UNSAFE` or `FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE` are used.
`FMOD_INIT_THREAD_UNSAFE` can be used safely if and only if: 1) only FMOD Studio
is used, and the Core API is never used; 2) FMOD Studio is not used, and Core
API calls all happen on a single thread; or 3) FMOD Studio is initialized with
`FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE` and all FMOD API calls are done in a
single thread. `FMOD_STUDIO_INIT_SYNCHRONOUS_UPDATE` can be safely if and only
if all Studio API calls happen on a single thread.<p>Thus, there are two ways to
make the FMOD.rs wrapper thread-safe: forbid the use of the thread-unsafe flags,
or encode thread safety into the type system. The former is trivial, but the
latter is potentially desirable for users who are using FMOD Studio with their
own asynchronous command queue. See the [Studio API Threads] for more on FMOD
Studio threading. Synchronous Update is very much a power-user feature and needs
to be encapsulated in a thread-safe worker queue to be used properly, though, so
it is acceptable if the thread-unsafe API notably less ergonomic to use than the
thread-safe API.<p>You might be tempted to thus just make `System` construction
`unsafe` and call that enough; let the user deal with it. However, a) that goes
against the Rust philosophy for typed correctness, as any "leakage" of FMOD.rs
types to code not aware of the giant caveat would then be unsound; and b) would
potentially still be unsound anyway if FMOD.rs API calls are still made from
safe code without knowledge that the library has been initialized in the
thread-unsafe mode.<p>Thus, exposing the thread-unsafe usage of FMOD is a
high-effort endeavor with minimal payoff; FMOD _permits_ thread-unsafe usage but
_recommends_ using the inbuilt thread-safe command batching.

[Studio API Threads]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-studio-threads.html
[Threads and Thread Safety]: https://fmod.com/resources/documentation-api?version=2.02&page=white-papers-threads.html
