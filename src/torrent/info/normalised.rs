use serde_bytes::ByteBuf;
use std::{io, path::PathBuf, sync::Arc};

use crate::torrent::{
    self, Info, InfoHash,
    info::{FileMode, InfoFile},
};

pub struct NormalisedInfo {
    pub name: String,
    pub(crate) piece_length: u32,
    pub pieces: ByteBuf,
    
    pub info_hash : InfoHash,

    pub total_length: u64,
    pub file_mode: FileMode,
}

impl NormalisedInfo {
    pub fn piece_len(&self, index: u32) -> u32 {
        let start = index as u64 * self.piece_length as u64;
        let end = (start + self.piece_length as u64).min(self.total_length);
        (end - start) as u32
    }

    /// Consumes `NormalizedInfo` to give an Arc<Self>
    pub fn atomic(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn piece_hash(&self, index: u32) -> &[u8; 20] {
        let start = index as usize * 20;
        let end = start + 20;
        self.pieces[start..end].try_into().expect(&format!(
            "InfoHash has unexpected size {size}, but 20 was expected",
            size = end - start
        ))
    }

    pub fn base_dir(&self) -> torrent::Result<PathBuf> {
        let base_dir = dirs::data_dir()
            .ok_or(torrent::Error::DataDirMissing)?
            .join(".qbit")
            .join(self.info_hash.to_string());
        Ok(base_dir)
    }
}

impl TryFrom<&Info> for NormalisedInfo {
    type Error = torrent::Error;
    fn try_from(info: &Info) -> Result<Self, Self::Error> {
        let file_mode = match (info.length, info.files.as_ref()) {
            (Some(length), None) => FileMode::Single {
                length: length as u64,
            },
            (None, Some(files)) => FileMode::Multiple {
                files: files.clone(),
            },
            _ => Err(torrent::Error::InvalidTorrent)?,
        };

        let total_length = match file_mode {
            FileMode::Single { length } => length,
            FileMode::Multiple { ref files } => files.iter().map(|f| f.length as u64).sum(),
        };

        let expected_pieces = (total_length + info.piece_length as u64) / info.piece_length as u64;
        let actual_pieces = info.pieces.len() / 20;

        if expected_pieces != actual_pieces as u64 {
            return Err(torrent::Error::InvalidTorrent);
        }

        Ok(Self {
            name: info.name.clone(),
            piece_length: info.piece_length,
            pieces: info.pieces.clone(),
            info_hash : info.info_hash,
            total_length,
            file_mode,
        })
    }
}
