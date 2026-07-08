Returns information on the memory usage of FMOD.

This information is byte accurate and counts all allocs and frees
internally. This is useful for determining a fixed memory size to make
FMOD work within for fixed memory machines such as consoles.

Note that if using
[`memory::initialize`],
the memory usage will be slightly higher than without it, as FMOD has to
have a small amount of memory overhead to manage the available memory.