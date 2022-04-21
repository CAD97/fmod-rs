//! This module documents platform-specific details on using FMOD.

#[cfg(doc)]
use fmod::*;

/// # Windows specific starter guide
///
#[doc = include_str!("Windows.md")]
pub mod windows {}

/// # macOS specific starter guide
///
#[doc = include_str!("macOS.md")]
pub mod macos {}
