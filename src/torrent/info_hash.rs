use std::{fmt::{Debug, Display}, ops::Deref};
use serde::Deserialize;


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
    pub fn to_url_encoded(&self) -> String {
        self.iter().map(|x| format!("%{x:02X}")).collect()
    }
}

impl Deref for InfoHash {
    type Target = [u8;20];
    fn deref(&self) -> &Self::Target {
        &self.hash
    }
}