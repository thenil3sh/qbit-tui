use std::io::Read;

use qbit::{
    torrent::Metadata,
    tracker,
};

#[tokio::main]
async fn main() {
    let torrent = Metadata::from_file("test/debian.torrent").unwrap();

    let url = tracker::get_url(&torrent);

    let response = reqwest::get(url)
        .await
        .expect("Failed to send request")
        .bytes()
        .await
        .expect("Failed to get message body");
    
    println!("{}", str::from_utf8(&response).unwrap());
    
    let resp : tracker::Response = bendy::serde::from_bytes(response.as_ref()).unwrap();
    // println!("{:?}", resp);
}
