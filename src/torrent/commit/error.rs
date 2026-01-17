use std::io;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error)
}

pub type Result<T> = std::result::Result<T, Error>;