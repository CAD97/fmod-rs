# FMOD.rs

Bindings to the [FMOD adaptive audio solution](https://fmod.com/).

[What is Adaptive Audio? (video)](https://youtu.be/p-FLWabby4Y)

## Licensing

This is up top because it's important: FMOD is _not_ free software! In order to
use FMOD, you must [acquire a license](https://www.fmod.com/sales). However, if
you make less than $200k revenue/year on a small (<$500k) development budget,
FMOD provides a [Free Indie License](https://www.fmod.com/sales#indie-note).

The Rust code in this repository is licensed under MIT OR Apache-2.0, but this
does not remove the requirement to comply with the FMOD license.

The examples in the [examples](examples) folder are direct ports from the
distributed FMOD examples, and are thus under the FMOD license. Similarly,
the [media](media) folder is also under the FMOD license.

## Providing the FMOD runtime

In order to better comply with the FMOD license, we do _not_ redistribute the
FMOD Engine. You should [download](https://www.fmod.com/download#fmodengine)
the engine yourself.

The currently vendored headers are for FMOD Engine 2.02.05 (build 124257). To
pin the headers to a specific version, pin `fmod-sys` and `fmod-studio-sys`.

We add `lib/{arch}` to the search path, and link the logging libraries for
development builds and production libraries for release builds. The dynamic
library is implicitly loaded from the run directory, or you can load them
explicitly.

### Windows Note

FMOD provides the 64 bit windows files in a `x64` folder; we use the convention
used in the linux distribution and by the Rust toolchain, and thus you need to
rename the arch folder to `x86_64`. Additionally, the `.lib` files have a `_vc`
suffix which need to be removed such that cargo/rustc can link them properly.

## Functionality

### Complete

- Raw bindings to the FMOD C API linking and running.
- Simplest `play_sound` example runs on wrapped API.
- Thread-safe API.

### Planned (Soon™)

- All examples using Rust idiomatic APIs.
- Test build and lib loading on macOS and Linux.

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
Studio threading. Syncrhonous Update is very much a power-user feature and needs
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
