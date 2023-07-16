use fmod::{raw::*, *};

opaque! {
    /// An interface that manages Digital Signal Processor (DSP) Connections.
    weak class DspConnection = FMOD_DSPCONNECTION, FMOD_DSPConnection_*;
}

// TODO: connections are invalidated when units are disconnected.
//       it is extremely unclear how to model this in Rust.
