use core::fmt::Debug;

use alloc::boxed::Box;

#[derive(Debug)]
pub enum Error {
    UnknownStep(u8),
    NoneTimer,
    UnexpectedPacket,
    NoSignature,

    /// Not a error, only timeout.
    Timeout,

    AppError(Box<dyn Debug>),
    NetworkError(Box<dyn Debug>),
}

impl Error {
    pub fn app_error(e: impl Debug + 'static) -> Self {
        Self::AppError(Box::new(e))
    }

    pub fn network_error(e: impl Debug + 'static) -> Self {
        Self::NetworkError(Box::new(e))
    }
}

pub type Result<T> = core::result::Result<T, Error>;
