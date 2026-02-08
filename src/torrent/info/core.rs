use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::{fmt::Debug, ops::Deref, sync::Arc};

use crate::torrent::{info::{FileMode, NormalisedInfo}, InfoHash};

/// # [`Info`]
/// A direct deserialized Info struct of Bencoded Torrent Metadata.\
/// Less efficient, and perfomance costing implementation is done with this struct. (unintentionally)\
/// A derived struct of [`Info`], called [`NormalisedInfo`]  should be considered in practice...
/// as it offers optimised computation and easier use.
#[derive(Deserialize)]
pub struct Info {
    pub(crate) name: String,
    #[serde(rename = "piece length")]
    pub(crate) piece_length: u32,

    pub(crate) pieces: ByteBuf,

    #[serde(flatten)]
    pub(crate) file_mode : FileMode,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct InfoFile {
    pub length: u64,
    pub path: Vec<String>,
}

pub type AtomicInfo = Arc<Info>;

impl Info {
    /// Returns specified piece's length. Costs
    /// - _O(1)_ time when it's a single file torrent
    /// - Otherwise _O(n)_ in case of multiple file torrent.
    /// 
    /// [`NormalisedInfo::piece_len`] handles both cases in _O(1)_ time, thus recommended to be taken in use.
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

    /// Returns total length of downloadable content, in bytes.
    pub fn total_length(&self) -> u64 {
        match self.file_mode.as_ref() {
            FileMode::Single { length } => *length,
            FileMode::Multiple { files } => files.iter().map(|f| f.length).sum(),
        }
    }

    /// Consumes info and gives out [`AtomicInfo`] (aliased [`Arc<Info>`])
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
