This is an unstructured dumping ground of ideas and notes around development.

### Thread Safety 2022-03-15

[`FMOD_System_Create`](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#system_create),
[`FMOD_System_Release`](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#system_release),
[`FMOD_Studio_System_Create`](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_create),
and [`FMOD_Studio_System_Release`](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_release)
are _very_ thread-unsafe? If these functions potentially race with _any other_ FMOD API, it's UB.

We really don't want to put the entirety of the FMOD API behind a RWLock just for this. Instead, we
take the approach of limiting the user to only (safely) creating one `System`, and marking the few
global system functions as `unsafe`. Thankfully, there's no requirement that `System` is released on
the same thread it was created on, so, as every other API function is predicated on the `System`
existing, nothing can race with its creation or release anymore.

Specifically, we add a global reference count for the initialized `System`, and every `Handle` owns
a strong reference count, keeping the `System` alive. This is a small amount of overhead, but I
believe that it's worth it for the usability benefits of not requiring every type with a `release`
to hold a `'system` lifetime.
