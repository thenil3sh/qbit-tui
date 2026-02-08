use serde::Deserialize;

use crate::torrent::info::InfoFile;

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum FileMode {
    Single { length: u64 },
    Multiple { files: Vec<InfoFile> },
}

impl AsRef<FileMode> for FileMode {
    fn as_ref(&self) -> &FileMode {
        self
    }
}