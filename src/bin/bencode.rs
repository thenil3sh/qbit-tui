use qbit::torrent::Metadata;



fn main() {
    let torrent = Metadata::from_file("test/debian.torrent").unwrap();
    println!("{torrent:#?}");
}