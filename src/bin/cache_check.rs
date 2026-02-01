use qbit::{torrent::Metadata, tracker::load_cache_or_fetch_tracker};

#[tokio::main]
async fn main () {
    let torrent = Metadata::from_file("test/debian.torrent").unwrap();
    let response = load_cache_or_fetch_tracker(&torrent).await.unwrap();

    println!("{response:?}");
}