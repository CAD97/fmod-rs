//! This module documents platform-specific details on using FMOD.

#[cfg(doc)]
use fmod::*;

#[doc = include_str!("Windows.md")]
pub mod windows {}

#[doc = include_str!("macOS.md")]
pub mod macos {}

#[doc = include_str!("Linux.md")]
pub mod linux {}
