use dirs::data_dir;
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::{fmt::Debug, ops::Deref, path::PathBuf, sync::Arc};

use crate::torrent::InfoHash;

#[derive(Deserialize)]
pub struct Info {
    pub(crate) name: String,
    #[serde(rename = "piece length")]
    pub(crate) piece_length: u32,
    #[serde(skip)]
    pub(crate) info_hash: InfoHash,
    pub(crate) pieces: ByteBuf,
    
    pub(crate) length: u32,
}

#[derive(Deserialize)]
struct InfoFile {
    length : u32,
    path : Vec<String>
}

pub type AtomicInfo = Arc<Info>;

impl Info {
    pub fn piece_len(&self, index: u32) -> u32 {
        let num_pieces = self.pieces.len() as u32 / 20;
        match index {
            x if num_pieces <= x => {
                panic!("Index out of bounds, length is {num_pieces}, but found {x}")
            }
            x if x == num_pieces - 1 => self.length % self.piece_length,
            _ => self.piece_length,
        }
    }

    /// Consumes info and gives out Arc<Info>
    pub fn atomic(self) -> AtomicInfo {
        Arc::new(self)
    }

    fn base_dir(&self) -> PathBuf {
        data_dir()
            .expect("Couldn't find data directory")
            .join(self.info_hash.to_string())
            .to_path_buf()
    }

    pub(crate) fn file_path(&self) -> PathBuf {
        let mut path = self.base_dir().join(self.name.as_str());
        path.set_extension("tmp");
        path
    }
}

#[derive(Default, Deserialize)]
pub struct RawInfo(pub Vec<u8>);

impl TryFrom<&[u8]> for Info {
    type Error = bendy::serde::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        bendy::serde::from_bytes(value)
    }
}

impl From<Vec<u8>> for RawInfo {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Deref for RawInfo {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<RawInfo> for Vec<u8> {
    fn from(value: RawInfo) -> Self {
        value.0
    }
}

impl Debug for RawInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ... ]")
    }
}
