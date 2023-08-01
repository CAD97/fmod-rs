(bindgen fmod.h
  --no-layout-tests
  --ctypes-prefix ::std::ffi
  --no-convert-floats
  --no-prepend-enum-name
  --raw-line "/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2023. */"
| str replace -as 'extern "C"' 'extern "system"'
| str replace -a  '(?s)extern "system" (fn\([^)]*\.\.\.[^)]*\))' 'extern "C" $1'
| str replace -as 'type_' 'r#type'
| str replace -as '__bindgen_anon_1' 'payload'
| str replace -as '__bindgen_ty_1' '_PAYLOAD'
| save -f bindings.rs)
(bindgen fmod_android.h --disable-header-comment --no-prepend-enum-name
| save -a bindings.rs)
(bindgen fmod_ios.h     --disable-header-comment --no-prepend-enum-name
| save -a bindings.rs)
(bindgen fmod_uwp.h     --disable-header-comment --no-prepend-enum-name
| save -a bindings.rs)
