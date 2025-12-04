use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::fmt::{Display, Debug};

#[derive(Deserialize)]
pub struct Info {
    pub length: u32,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: u32,
    pub pieces: ByteBuf,
}

#[derive(Deserialize)]
pub struct InfoHash {
    hash: [u8; 20],
}

impl Default for InfoHash {
    fn default() -> Self {
        InfoHash { hash: [0; 20] }
    }
}

impl From<[u8; 20]> for InfoHash {
    fn from(value: [u8; 20]) -> Self {
        InfoHash { hash: value }
    }
}

impl Display for InfoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash.iter().map(|x| format!("{x:0x}")).collect::<String>())
    }
}

impl Debug for InfoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.to_string())
    }
}

impl InfoHash {
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }
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
