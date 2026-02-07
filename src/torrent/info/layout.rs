use std::{path::{Path, PathBuf}, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::torrent::{self, info::{FileMode, NormalisedInfo}};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileLayout {
    pub(crate) files: Vec<FileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FileEntry {
    pub(crate) path: PathBuf,
    pub(crate) length: u64,
    pub(crate) offset: u64,
}

pub type AtomicFileLayout = Arc<FileLayout>;

impl FileLayout {
    pub(crate) fn build(base_path : &Path, info: &NormalisedInfo) -> Self {
        let mut entries= Vec::new();
        let mut offset = 0u64;

        match info.file_mode.as_ref() {
            FileMode::Single { length } => {
                let file = FileEntry {
                    path : base_path.join(&info.name).with_extension("tmp"),
                    length : *length,
                    offset,
                };
                entries.push(file);
            }
            FileMode::Multiple { files } => {
                for f in files {
                    let file = FileEntry {
                        path : f.path.iter().fold(base_path.to_owned(), |prev, current| prev.join(current)).with_extension("tmp"),
                        length : f.length,
                        offset,
                    };
                    entries.push(file);
                    offset += f.length;
                }
            }
        }
        
        Self { files : entries }
    }


    /// **Consumes** FileLayout and returns an Arc<FileLayout>
    pub(crate) fn atomic(self) -> AtomicFileLayout {
        Arc::new(self)
    }
}

impl TryFrom<&NormalisedInfo> for FileLayout {
    type Error = torrent::Error;
    fn try_from(info: &NormalisedInfo) -> Result<Self, Self::Error> {
        let base_dir = info.base_dir()?;
        Ok(Self::build(&base_dir, info))
    }
}