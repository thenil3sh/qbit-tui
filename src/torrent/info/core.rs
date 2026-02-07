use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::{fmt::Debug, ops::Deref, sync::Arc};

use crate::torrent::InfoHash;

#[derive(Deserialize)]
pub struct Info {
    pub(crate) name: String,
    #[serde(rename = "piece length")]
    pub(crate) piece_length: u32,

    pub(crate) pieces: ByteBuf,

    #[serde(skip)]
    pub(crate) info_hash: InfoHash,

    pub(crate) length: Option<u32>,
    pub(crate) files: Option<Vec<InfoFile>>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct InfoFile {
    pub length: u64,
    pub path: Vec<String>,
}

pub type AtomicInfo = Arc<Info>;

impl Info {
    pub fn piece_len(&self, index: u32) -> u32 {
        let num_pieces = self.pieces.len() as u32 / 20;
        if num_pieces <= index {
            panic!("Index out of bounds, length is {num_pieces}, but found {index:?}");
        }
        let total_length = self.total_length();
        let start = index as u64 * self.piece_length as u64;
        let end = (start + self.piece_length as u64).min(total_length);
        (end - start) as u32
    }

    pub fn total_length(&self) -> u64 {
        match (self.length, self.files.as_ref()) {
            (Some(length), None) => length as u64,
            (None, Some(files)) => files.iter().map(|f| f.length).sum(),
            (_, _) => panic!("Invalid torrent file"),
        }
    }

    /// Consumes info and gives out Arc<Info>
    pub fn atomic(self) -> AtomicInfo {
        Arc::new(self)
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
