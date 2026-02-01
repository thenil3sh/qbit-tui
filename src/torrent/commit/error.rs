use std::io;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Base Directory couldn't be found")]
    BaseDirectoryNotFound
}

pub type Result<T> = std::result::Result<T, Error>;