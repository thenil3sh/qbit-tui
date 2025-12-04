use std::{fs, io::{read_to_string, Read, Write}};
use sha1::{digest::crypto_common::KeyInit, Digest, Sha1};
use qbit::torrent::Metadata;
use bendy::decoding::Object::{self, *};
use qbit::torrent::Info;

fn main() {
    let torrent = Metadata::from_file("test/oreo.torrent").unwrap();
    println!("{torrent:#?}");
}