use std::{sync::Arc, time::Duration};

use qbit::{
    peer::{Handshake, PeerSession},
    torrent::{self, Committer, Metadata, State},
    tracker::{self},
};
use tokio::{sync::Mutex, task::JoinSet, time::timeout};

#[tokio::main]
async fn main() {
    let torrent = Arc::new(
        Metadata::from_file("test/debian.torrent").expect("Fucking failed at reading torrent"),
    );

    let torrent_info: Arc<torrent::Info> = Arc::new(torrent.info_byte().try_into().unwrap());
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::load_or_new(&torrent).await));
    let peers: tracker::Response = tracker::load_cache_or_fetch_tracker(&torrent)
        .await
        .expect("Failed fetching tracker")
        .try_into()
        .expect("Failed parsing tracker's response into struct");

    let connection_list = Arc::new(Mutex::new(Vec::new()));
    let mut join_set = JoinSet::new();
    let mut committer = Committer::new(state.clone(), torrent.info_hash, torrent_info.clone());

    for (index, &peer) in peers.peers.iter().enumerate() {
        let handshake = Handshake::new(&torrent.info_hash);
        let timeout_session = timeout(Duration::from_secs(10), {
            let connection_list = connection_list.clone();
            async move {
                if let Ok(mut connection) = peer.connect().await {
                    if let Ok(()) = connection.handshake(handshake).await {
                        eprintln!("Peer {index} handshake success!!");
                        let mut connection_list = connection_list.lock().await;
                        connection_list.push(connection);
                    }
                }
            }
        });
        join_set.spawn(async move {
            if timeout_session.await.is_err() {
                println!("Peer {index} : Timed out");
            }
        });
    }
    join_set.join_all().await;

    println!(
        "\x1b[32m{} peers responded, in total\x1b[0m",
        connection_list.clone().lock().await.len()
    );
    
    let connection_list = {
        let mut guard = connection_list.lock().await;
        std::mem::take(&mut *guard)
    };
    
    let mut join_set = JoinSet::new();
    let count = Arc::new(Mutex::new(0usize));
    for i in connection_list {
        join_set.spawn({
            let state = state.clone();
            let torrent_info = torrent_info.clone();
            let count = count.clone();
            let receiver = committer.listener();
            let sender = committer.sender().clone();
            async move {
                let mut session = PeerSession::new(i, torrent_info, state, sender, receiver);
                if let Err(x) = session.run().await {
                    eprintln!("\x1b[033mSession Error {x:?}\x1b[0m");
                }
                *count.lock().await += 1;
            }
        });
    }
    
    join_set.spawn(async move {
        committer.run().await.unwrap();
    });

    join_set.join_all().await;
    eprintln!("All connections closed, {}, failed", count.lock().await);
}
