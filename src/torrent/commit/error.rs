use std::io;
use tokio::sync::broadcast::error::SendError;

use crate::torrent::commit;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Base Directory couldn't be found")]
    BaseDirectoryNotFound,
    #[error(transparent)]
    SendErr(#[from] SendError<commit::Event>),
}

pub type Result<T> = std::result::Result<T, Error>;
