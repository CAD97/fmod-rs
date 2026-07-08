Specify the level and delivery method of log messages when using the
logging version of FMOD.

This function will return
[`Error::Unsupported`]
when using the non-logging (release) versions of FMOD.

The logging version of FMOD can be recognized by the \'L\' suffix in the
library name, fmodL.dll or libfmodL.so for instance.

Note that:

- [`DebugFlags::LevelLog`]
  produces informational, warning and error messages.
- [`DebugFlags::LevelWarning`]
  produces warnings and error messages.
- [`DebugFlags::LevelError`]
  produces error messages only.