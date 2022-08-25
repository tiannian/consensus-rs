pub enum Error {
    UnknownStep(u8),
}

pub type Result<T> = core::result::Result<T, Error>;

