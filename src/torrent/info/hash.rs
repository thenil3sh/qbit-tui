use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

#[derive(Default, Deserialize, Clone, Copy, Serialize)]
pub struct InfoHash {
    hash: [u8; 20],
}

impl From<[u8; 20]> for InfoHash {
    fn from(value: [u8; 20]) -> Self {
        InfoHash { hash: value }
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
