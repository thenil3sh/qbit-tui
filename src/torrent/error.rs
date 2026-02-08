#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Torrent parse failed")]
    InvalidTorrent,
    #[error(transparent)]
    DeserialisationError(#[from] bendy::serde::Error),
    #[error("Data directory missing or not set")]
    DataDirMissing
}

pub type Result<T> = std::result::Result<T, Error>;