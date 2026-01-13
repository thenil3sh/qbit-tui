#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Torrent parse failed")]
    InvalidTorrent,
}