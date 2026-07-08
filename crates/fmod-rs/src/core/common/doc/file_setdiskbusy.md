Sets the busy state for disk access ensuring mutual exclusion of file
operations.

If file IO is currently being performed by FMOD this function will block
until it has completed.

This function should be called in pairs once to set the state, then
again to clear it once complete.