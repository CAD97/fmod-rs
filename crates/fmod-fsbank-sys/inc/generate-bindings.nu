(bindgen fsbank.h
  --no-prepend-enum-name
  --raw-line "/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2023. */"
| str replace -a  'extern "C"' 'extern "system"'
| str replace -a  'type_' 'r#type'
| str replace -a  '__bindgen_anon_1' 'payload'
| str replace -a  '__bindgen_ty_1' '_PAYLOAD'
| save -f --raw bindings.rs)
