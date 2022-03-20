# Every other FMOD binding is unsound.

No, that's not exagerating for clickbait; it's true. It's an edge case, but Rust's safety rules don't allow for edge cases without saying `unsafe`.

Creating an [FMOD Core System](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#system_create)
or an [FMOD Studio System](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_create)
is thread-unsafe to _any other FMOD API call_ (including themselves). To make matters even more "interesting",
[releasing](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#system_release)
[them](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_release)
is thread-unsafe as well!

Even more fun, there's a flag that
[explicitly requests everything to be thread unsafe](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#fmod_init_thread_unsafe).

No FMOD binding that [I can find](https://lib.rs/search?q=fmod) protects against this unsoundness.
Even assuming every other API surface is wrapped soundly, every FMOD existing binding is unsound.

This crate takes a simple enough approach, which is decently common for FFI wrappers:
control `System` creation, so that only one is ever created. All owned library handles own a reference count to the global `System`.
This isn't 100% zero-cost — resources could be instead bound to the lifetime of the `System` — but it is much easier to work with.

If desired, I can add a `RefHandle` to put that lifetime on which uses +0 reference counting to make it near-zero-cost.
The only remaining cost would be the global lock and pointer on the `System` singleton, which is unavoidable in a sound API.
