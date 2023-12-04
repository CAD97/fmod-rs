(bindgen fmod_studio.h
  --no-layout-tests
  --no-recursive-allowlist
  --ctypes-prefix ::std::ffi
  --no-convert-floats
  --no-prepend-enum-name
  --raw-line "/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2023. */"
  # blocklist fmod-core-sys items
  --allowlist-file fmod_studio.h
  --allowlist-file ./fmod_studio_common.h
  # in a union, so needs to be allowlisted
  --allowlist-type FMOD_BOOL
  --merge-extern-blocks
  # re-add derives skipped due to blocklisting
  --with-derive-custom FMOD_STUDIO_BANK_INFO=Debug,Copy,Clone
  --with-derive-custom FMOD_STUDIO_PARAMETER_DESCRIPTION=Debug,Copy,Clone
  --with-derive-custom FMOD_STUDIO_TIMELINE_NESTED_BEAT_PROPERTIES=Debug,Copy,Clone
  --with-derive-custom FMOD_STUDIO_SOUND_INFO=Debug,Copy,Clone
| str replace -arm '^.*type FMOD_BOOL.*$\n' ''
| str replace -a   'extern "C"' 'extern "system"'
| str replace -a   'type_' 'r#type'
| str replace -a   '__bindgen_anon_1' 'payload'
| str replace -a   '__bindgen_ty_1' '_PAYLOAD'
| save bindings.rs -f)
