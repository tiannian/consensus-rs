//! Error

use core::fmt::Debug;

use alloc::boxed::Box;

/// Error
#[derive(Debug)]
pub enum Error {
    /// Got a unexpected packet from network
    UnexpectedPacket,
    /// Lose signature on packet
    NoSignature,

    /// Not a error, only timeout
    Timeout,

    /// Error from app
    AppError(Box<dyn Debug>),

    /// Error from network
    NetworkError(Box<dyn Debug>),
}

impl Error {
    pub(crate) fn app_error(e: impl Debug + 'static) -> Self {
        Self::AppError(Box::new(e))
    }

    pub(crate) fn network_error(e: impl Debug + 'static) -> Self {
        Self::NetworkError(Box::new(e))
    }
}

/// Alias of crate result.
pub type Result<T> = core::result::Result<T, Error>;
