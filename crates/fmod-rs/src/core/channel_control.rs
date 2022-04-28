use fmod::*;

/// A trait for the shared functionality between [Channel] and [ChannelGroup].
pub trait ChannelControl: Sealed {}

impl Sealed for Channel {}
impl ChannelControl for Channel {}

impl Sealed for ChannelGroup {}
impl ChannelControl for ChannelGroup {}

use sealed::Sealed;
mod sealed {
    pub trait Sealed {}
}
