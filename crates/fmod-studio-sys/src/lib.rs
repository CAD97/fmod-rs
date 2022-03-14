#![allow(
    deref_nullptr,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]

use fmod_core_sys::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
