use crate::torrent::{Info, InfoHash};
use anyhow::{anyhow, bail};
use bendy::decoding::Object::Dict;
use serde::Deserialize;
use std::{fs, path::Path};
use sha1::{Sha1, Digest};
use crate::torrent::RawInfo;

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub announce: String,
    #[serde(rename = "created by")]
    pub created_by: String,
    #[serde(rename = "creation date")]
    pub creation_date: usize,

    pub info: Info,

    #[serde(default)]
    pub info_hash: InfoHash,

    #[serde(default)]
    info_byte: RawInfo,

    #[serde(rename = "url-list")]
    pub url_list: Vec<String>,
}

impl Metadata {
    pub fn from_file<T: AsRef<Path>>(file: T) -> Result<Self, anyhow::Error> {
        let file = fs::read(file.as_ref())?;

        let mut metadata: Self =
            bendy::serde::from_bytes(&file).expect("Failed to parse torrent into TorrentMeta");

        metadata.info_byte = Self::scrap_raw_info(&file).unwrap().into();
        metadata.info_hash = Self::generate_info_hash(&metadata.info_byte).unwrap().into();

        Ok(metadata)
    }

    pub fn info_byte(&self) -> &[u8] {
        self.info_byte.as_ref()
    }
    
    fn generate_info_hash(buffer: &[u8]) -> Result<[u8; 20], anyhow::Error> {
        let mut hasher = Sha1::new();
        hasher.update(buffer);
        Ok(hasher.finalize().into())
    }

    fn scrap_raw_info(buffer: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let mut decoder = bendy::decoding::Decoder::new(buffer);

        let torrent_object = decoder
            .next_object()?
            .ok_or_else(|| anyhow!("Empty torrent file?"))?;
        let mut dict = match torrent_object {
            Dict(x) => x,
            _ => bail!("Invalid torrent format, expected a bencoded dictionary structure"),
        };

        while let Ok(Some(object)) = dict.next_pair() {
            if let (b"info", Dict(x)) = object {
                return Ok(x.into_raw()?.into());
            }
        }
        bail!("Invalid torrent format : missing info dictionary")
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

impl Info {}

mod test {
    #[allow(unused_imports)]
    use super::*;
    
}