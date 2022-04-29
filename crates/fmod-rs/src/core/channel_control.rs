use fmod::*;

/// A trait for the shared functionality between [Channel] and [ChannelGroup].
//
//  NB: this is replicated as part of the FMOD API, but while the C headers have
//  FMOD_CHANNEL_CONTROL, all ChannelControl C functions (claim to be) specific
//  to Channel or ChannelGroup.
pub trait ChannelControl: Sealed {}

impl Sealed for Channel {}
impl ChannelControl for Channel {}

impl Sealed for ChannelGroup {}
impl ChannelControl for ChannelGroup {}

use sealed::Sealed;
mod sealed {
    pub trait Sealed {}
}
