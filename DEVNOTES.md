This is an unstructured dumping ground of ideas and notes around development.

### Thread Safety 2022-03-15

Why must [`FMOD_System_Create`](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#system_create),
[`FMOD_System_Release`](https://fmod.com/resources/documentation-api?version=2.02&page=core-api-system.html#system_release),
[`FMOD_Studio_System_Create`](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_create),
and [`FMOD_Studio_System_Release`](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_release)
be _this_ thread-unsafe? If these functions potentially race with _any other_ FMOD API, it's UB.
This means that _dropping_ `fmod::Handle<fmod::System>` or `fmod::Handle<fmod::studio::System>` is
potentially thread-unsafe, which is very unfortunate.

We really don't want to put the entirety of the FMOD API behind a RWLock just for this. Instead, we
take the approach of limiting the user to only (safely) creating one `System`, and marking the few
global system functions as `unsafe`. Thankfully, there's no requirement that `System` is released on
the same thread it was created on, so, as every other API function is predicated on the `System`
existing, nothing can race with its creation or release anymore

oh no oh wait that's not true, e.g. `Handle<Sound>` calls `FMOD_Sound_*` APIs without requiring the
`System` to chime in so it can race with the `System` and even be used afterwards and oh no I'm going
to have to reference count the `System` in order to maintain soundness :(
