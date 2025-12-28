use qbit::torrent::Metadata;

fn main() {
    let torrent = Metadata::from_file("test/oreo.torrent").unwrap();
    println!("{torrent:#?}");

    println!("{:?}", torrent.info_hash);
}