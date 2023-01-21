//! Error

use core::fmt::Debug;

use crate::Core;

/// Error
#[derive(Debug)]
pub enum Error<C: Core> {
    /// Got a unexpected packet from network
    UnexpectedPacket,
    /// Lose signature on packet
    NoSignature,

    /// Not a error, only timeout
    Timeout,

    /// Error from app
    ConsensusError(C::Error),
}

/// Alias of crate result.
pub type Result<C, T> = core::result::Result<T, Error<C>>;
