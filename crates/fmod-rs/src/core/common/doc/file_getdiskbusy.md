Information function to retrieve the state of FMOD disk access.

Do not use this function to synchronize your own reads with, as due to
timing, you might call this function and it says false = it is not busy,
but the split second after calling this function, internally FMOD might
set it to busy. Use
[`file::set_disk_busy`] for
proper mutual exclusion as it uses semaphores.