use std::{fs, io, path::Path};
use serde::{Deserialize};
use serde_bytes::ByteBuf;

#[derive(Debug, Deserialize)]
pub struct TorrentMeta {
    pub announce : String,
    #[serde(rename = "created by")]
    pub created_by : String,
    #[serde(rename = "creation date")]
    pub creation_date : usize,
    pub info : Info,
}

#[derive(Deserialize)]
pub struct Info {
    pub length : u32,  
    pub name : String,
    #[serde(rename = "piece length")]
    pub piece_length : u32,
    pub pieces : ByteBuf,
    // pub pieces : String
    // pub info_hash : [u8; 20]
}

impl TorrentMeta {
    pub fn from_file<T : AsRef<Path>>(file : T) -> Result<Self, io::Error> {
        let file = fs::read(file.as_ref())?;

        let bencode: TorrentMeta  = bendy::serde::from_bytes(&file)
        .expect("Failed to parse torrent into TorrentMeta");

        Ok(bencode)
    }
}

use std::fmt::Debug;
impl Debug for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Info")
            .field("length", &format!("{} MBs", self.length))
            .field("name", &self.name)
            .field("piece_size", &self.piece_length)
            .field("pieces", &"[ ... ]")
            .finish()
    }
}