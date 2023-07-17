//! Platform Details

#[cfg(doc)]
use fmod::*;

#[doc = include_str!("Windows.md")]
pub mod windows {}

#[doc = include_str!("macOS.md")]
pub mod macos {}

#[doc = include_str!("Linux.md")]
pub mod linux {}

#[doc = include_str!("iOS.md")]
pub mod ios {}

#[doc = include_str!("Android.md")]
pub mod android {}

#[doc = include_str!("UWP.md")]
pub mod uwp {}
