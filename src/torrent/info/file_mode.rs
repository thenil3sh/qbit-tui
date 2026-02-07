use crate::torrent::info::InfoFile;

pub enum FileMode {
    Single { length: u64 },
    Multiple { files: Vec<InfoFile> },
}

impl AsRef<FileMode> for FileMode {
    fn as_ref(&self) -> &FileMode {
        self
    }
}