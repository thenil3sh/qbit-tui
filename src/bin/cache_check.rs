use std::time::Duration;

use qbit::{torrent::Metadata, tracker::load_cache_or_fetch_tracker};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let torrent = Metadata::from_file("test/debian.torrent").unwrap();
    let response = load_cache_or_fetch_tracker(&torrent).await.unwrap();

    println!("{response:?}");

    let (sender, _) = tokio::sync::broadcast::channel::<u64>(4);

    let mut joinset = JoinSet::new();

    joinset.spawn({
        let mut reciever = sender.subscribe();
        async move {
            let x = reciever.recv().await.unwrap();
            eprintln!("Recieved {x}");
        }
    });

    joinset.spawn({
        let mut reciever = sender.subscribe();
        async move {
            let x = reciever.recv().await.unwrap();
            eprintln!("Recieved two {x}");
        }
    });
    joinset.spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = sender.send(69);
        eprintln!("Sent 69");
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _ = sender.send(69);
        eprintln!("Sent 69");
    });

    joinset.join_all().await;
}
