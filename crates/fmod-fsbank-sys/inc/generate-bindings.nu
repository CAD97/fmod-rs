(bindgen fsbank.h
  --no-prepend-enum-name
  --raw-line "/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2023. */"
| str replace -as 'extern "C"' 'extern "system"'
| str replace -as 'type_' 'r#type'
| str replace -as '__bindgen_anon_1' 'payload'
| str replace -as '__bindgen_ty_1' '_PAYLOAD'
| save -f --raw bindings.rs)