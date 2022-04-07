This is an unstructured dumping ground of ideas and notes around development.

### Multiple `System` support 2022-04-06

One of the FMOD Core examples is using multiple systems. This seems to be the way that you play
sounds to multiple output devices. So, it'd be nice to support.

Problem 1) Thread safety. It's not thread safe to create or release the systems when _any_ FMOD API
could race. Solutions: limit the world to only one `System`, put an unnecessary global rwlock on
*everything*, or make create and release `unsafe` (and because release is `unsafe`, drop would be a
leak or a soundness hole). Note that giving everything a `'system` lifetime isn't enough to avoid
the race, as a `System` release could race with another system's use.

Problem 2) Everything becomes invalidated when `System` is released.

> All handles or pointers to objects associated with a Studio System object become invalid when the
> Studio System object is released. The FMOD Studio API attempts to protect against stale handles
> and pointers being used with a different Studio System object but this protection cannot be
> guaranteed and attempting to use stale handles or pointers may cause undefined behavior.
> [[Studio::System::release]](https://fmod.com/resources/documentation-api?version=2.02&page=studio-api-system.html#studio_system_release)

Similar language exists for a core system. Solutions: give everything a `'system` lifetime, or
reference count the system. Problems with reference counting: I *really* want to deal in `&Resource`
and not `&Handle<Resource>` (avoid the double indirection) as much as possible. As such, we need
some way to go from `&Resource` to `Handle<NewResource>`. Solutions: have a global way to create a
`Handle` (this means a global reference to the system's reference count) or store a reference to the
reference count in userdata. Problem: channels need to participate in this scheme somehow, and can't
use owned user data. Problem with a `'system` lifetime: while it *would* be possible to have e.g.
`&'system Owned<Sound>` rather than `Handle<'system, Sound>`, systems like bevy resources *really*
want to have `T: 'static`, and making a bevy plugin is the end goal of me working on these bindings.

The combination of these two things make for an annoying problem to solve. The only real workable
solutions that maintain soundness and usability I see are:

- Restrict the world to a single `System`, globally accessible reference count.
- Restrict the world to a single `System`, don't ever release it.
- ... see below.

Trying to enumerate the solution space, I think I finally came up with a good solution:

- `System::new` checks a global flag, panicking if a system already exists.
- `System::new_again` is `unsafe`, bypassing the above check.
- `type Handle<Resource> = &'static Owned<Resource>`.
- Releasing the system is `unsafe`.

This makes the common case safe; if you're just going to close the app anyway, releasing memory is
unnecessary. (In fact, letting the OS do it as one big delayed chunk can be better in many cases!)
It has *zero* overhead over using the API directly, as we don't add any extra state to maintain,
other than a single atomic check at startup. And the use case of multiple systems is still possible,
*and* we don't take up the user data slot from the end user. This feels like the best solution.

### `GetUserData` 2022-04-06

Don't put owned data in `FMOD_Channel_SetUserData`. When a channel is stolen, you can't get
the data anymore, and so it's going to leak. When a channel finishes playing, you can't get the data
anymore, and so it's going to leak. If a type doesn't have a `release`, assume it can be stolen and
you won't be able to recover any user data you give it. Only use unowned user data for these types.

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
