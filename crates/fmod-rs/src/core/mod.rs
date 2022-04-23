#![doc = include_str!("README.md")]

#[cfg(doc)]
use fmod::{raw, studio};

mod channel;
mod channel_group;
mod common;
mod dsp;
mod sound;
mod system;

pub use self::{channel::*, channel_group::*, common::*, dsp::*, sound::*, system::*};
