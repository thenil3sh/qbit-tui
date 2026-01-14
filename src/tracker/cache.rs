use std::{fs::{self, File}, io, path::Path};

use crate::torrent::{InfoHash, info};


pub(crate) fn update_cache(info_hash: &InfoHash, ) {
    let path = format!("~/.cache/qbit-tui/tracker/{}.bencode", info_hash.to_hex_lower());

    let file = fs::OpenOptions::new().write(true).truncate(true).open(path).expect("Couldn't make file");

}