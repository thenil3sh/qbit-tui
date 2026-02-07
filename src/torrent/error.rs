#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Torrent parse failed")]
    InvalidTorrent,
    #[error("Data directory missing or not set")]
    DataDirMissing
}

pub type Result<T> = std::result::Result<T, Error>;