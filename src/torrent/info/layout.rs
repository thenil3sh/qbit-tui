use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileLayout {
    files : Vec<FileEntry>
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    path : PathBuf,
    length : u32,
    offset : u32,
}