use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::fmt::Debug;

#[derive(Deserialize)]
pub struct Info {
    pub length: u32,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: u32,
    pub pieces: ByteBuf,
}

#[derive(Default, Deserialize)]
pub struct RawInfo (pub Vec<u8>);

impl From<Vec<u8>> for RawInfo {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
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