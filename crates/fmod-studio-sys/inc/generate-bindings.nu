# NB: rust-lang/rust-bindgen#2187 means that blocklisting is failing, so it has to be done by hand.
# Thankfully, the blocklisted items are a simple prefix of the emitted file.
(bindgen fmod_studio.h
  --no-prepend-enum-name
  --raw-line "/* Copyright (c), Firelight Technologies Pty, Ltd. 2004-2022. */"
  --blocklist-file fmod.h
  --blocklist-file fmod_codec.h
  --blocklist-file fmod_common.h
  --blocklist-file fmod_dsp.h
  --blocklist-file fmod_dsp_effects.h
  --blocklist-file fmod_errors.h
  --blocklist-file fmod_output.h
| str replace -as 'extern "C"' 'extern "system"'
| str replace -as 'type_' 'r#type'
| str replace -as '__bindgen_anon_1' 'payload'
| str replace -as '__bindgen_ty_1' '_PAYLOAD'
| save --raw bindings.rs -f)
