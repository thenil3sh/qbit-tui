use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use crate::torrent::RawInfo;

#[derive(Default, Deserialize, Clone, Copy, Serialize)]
pub struct InfoHash {
    hash: [u8; 20],
}

impl From<[u8; 20]> for InfoHash {
    fn from(value: [u8; 20]) -> Self {
        InfoHash { hash: value }
    }
}

impl From<&RawInfo> for InfoHash {
    fn from(raw_info: &RawInfo) -> Self {
        let mut hasher = Sha1::new();
        let buffer: &[u8] = raw_info.as_ref();
        hasher.update(buffer);
        let hash: [u8; 20] = hasher.finalize().into();
        InfoHash { hash }
    }
}

impl Display for InfoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.hash
                .iter()
                .map(|x| format!("{x:0x}"))
                .collect::<String>()
        )
    }
}

impl Debug for InfoHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self)
    }
}

impl InfoHash {
    pub fn to_url_encoded(&self) -> String {
        self.as_ref().iter().map(|x| format!("%{x:02X}")).collect()
    }

    pub fn to_hex_lower(&self) -> String {
        self.as_ref().iter().map(|x| format!("{x:x}")).collect()
    }
}

impl Deref for InfoHash {
    type Target = [u8; 20];
    fn deref(&self) -> &Self::Target {
        &self.hash
    }
}

impl AsRef<[u8]> for InfoHash {
    fn as_ref(&self) -> &[u8] {
        &self.hash
    }
}
