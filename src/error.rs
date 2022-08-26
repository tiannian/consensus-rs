pub enum Error {
    UnknownStep(u8),
    NoneTimer,
}

pub type Result<T> = core::result::Result<T, Error>;
