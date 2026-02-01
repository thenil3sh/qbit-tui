use qbit::{peer::Handshake, torrent, tracker};
use std::time::Duration;
use tokio::task::JoinSet;

// Tracker responds with this bencode...
// use ./beat-a-tracker.rs to generate a new one
static EXAMPLE_BENCODE : &[u8] = b"d8:intervali900e5:peersld2:ip13:146.70.182.124:porti46271eed2:ip13:158.173.3.1344:porti40841eed2:ip12:108.243.2.784:porti6882eed2:ip12:77.33.175.704:porti51413eed2:ip15:111.108.157.1394:porti51413eed2:ip14:146.70.226.2334:porti62975eed2:ip13:176.113.74.864:porti57020eed2:ip11:95.31.15.124:porti55612eed2:ip13:193.33.56.1354:porti51413eed2:ip11:67.225.4.954:porti58946eed2:ip14:37.120.213.2214:porti6881eed2:ip14:178.162.174.924:porti20010eed2:ip12:176.102.77.94:porti50413eed2:ip12:93.120.161.64:porti51413eed2:ip13:79.192.33.1414:porti51413eed2:ip12:47.26.207.224:porti33786eed2:ip12:37.48.89.2164:porti62803eed2:ip11:94.32.93.234:porti50413eed2:ip13:82.66.145.2294:porti50000eed2:ip12:38.59.26.2464:porti51413eed2:ip13:79.127.136.374:porti60102eed2:ip14:185.65.134.1804:porti45333eed2:ip13:90.153.49.1724:porti6952eed2:ip13:185.56.20.2024:porti54857eed2:ip13:85.130.239.934:porti6881eed2:ip11:92.63.31.994:porti41150eed2:ip14:37.110.206.1174:porti35831eed2:ip13:146.70.86.1164:porti6881eed2:ip12:84.175.8.2294:porti51414eed2:ip12:193.25.5.1594:porti51413eed2:ip14:152.53.109.2254:porti6881eed2:ip13:23.226.86.1974:porti8999eed2:ip12:64.99.246.234:porti51413eed2:ip11:41.71.3.1074:porti34575eed2:ip14:96.241.233.2424:porti51413eed2:ip14:130.44.142.1704:porti26340eed2:ip14:188.243.196.624:porti51413eed2:ip13:84.21.168.2324:porti6881eed2:ip14:176.15.201.2404:porti27481eed2:ip14:38.156.230.1514:porti16172eed2:ip12:176.3.109.824:porti62222eed2:ip14:90.255.244.2194:porti51234eed2:ip13:87.182.121.594:porti56886eed2:ip15:104.193.135.1174:porti51413eed2:ip14:24.162.210.1024:porti20443eed2:ip13:195.78.54.1614:porti24316eed2:ip13:188.114.67.534:porti41554eed2:ip13:185.203.56.274:porti57922eed2:ip13:190.153.80.364:porti51413eed2:ip15:180.154.114.1464:porti41683eeee";

// This will make handshake to all those peer's ip...(parallely) and log their responses
// One thing, I should care, actually two things
// - api is yet to be implemented
// - please run this less frequently, you don't wanna disturb those peers
#[tokio::main]
async fn main() {
    let info_hash = torrent::Metadata::from_file("test/oreo.torrent").unwrap().info_hash;

    let tracker_response: tracker::Response = EXAMPLE_BENCODE.try_into().unwrap();

    let mut handshakes = JoinSet::new();

    for (i, peer) in tracker_response.peers.iter().enumerate() {
        handshakes.spawn(handle_handshake(i, *info_hash, *peer));
    }

    handshakes.join_all().await;
}

async fn handle_handshake(i : usize,info_hash: [u8; 20], peer: qbit::peer::Peer) {
    let timeout = tokio::time::timeout(Duration::from_secs(5), async move {
        let handshake = Handshake::from(&info_hash);
        match peer.connect().await {
            Ok(mut connection) => {
                eprintln!("Waiting for peer {i:2}...");
                if let Ok(()) = connection.handshake(handshake).await {
                    eprintln!("\x1b[032mPeer {i:2} didn't refuse!!\x1b[0m");
                }
            }
            Err(x) => eprintln!("\x1b[033mPeer {i} failed to connect : {x}\x1b[0m")
            
        }
    })
    .await;
    if timeout.is_err() {
        eprintln!("\x1b[031mPeer : {i:2}, timed out\x1b[0m");
    }
}
