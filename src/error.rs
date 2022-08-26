pub enum Error {
    UnknownStep(u8),
    NoneTimer,
    UnexpectedPacket,
}

pub type Result<T> = core::result::Result<T, Error>;
