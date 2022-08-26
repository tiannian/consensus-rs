use core::fmt::Debug;

use alloc::boxed::Box;

#[derive(Debug)]
pub enum Error {
    UnknownStep(u8),
    NoneTimer,
    UnexpectedPacket,
    AppError(Box<dyn Debug>),
}

impl Error {
    pub fn from_core_debug(e: impl Debug + 'static) -> Self {
        Self::AppError(Box::new(e))
    }
}

pub type Result<T> = core::result::Result<T, Error>;
