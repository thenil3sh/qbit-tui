#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Protocol violation")]
    ProtocolViolation,
    #[error("Session timed out")]
    TimeOut,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}