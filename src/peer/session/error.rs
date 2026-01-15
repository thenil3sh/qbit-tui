#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Protocol violation")]
    ProtocolViolation,
    #[error("Bad Request")]
    BadRequest,
    #[error("Session timed out")]
    TimeOut,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;