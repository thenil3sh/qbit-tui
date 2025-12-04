use qbit::torrent::metadata::TorrentMeta;


fn main() {
    let torrent = TorrentMeta::from_file("oreo.torrent").unwrap();
    println!("{torrent:#?}");
}