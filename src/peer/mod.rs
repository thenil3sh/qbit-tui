pub mod id;
mod handshake;
mod connection;

use std::net::Ipv4Addr;

pub use id::PEER_ID as ID;
use serde::Deserialize;
pub use handshake::Handshake;
pub use connection::Connection as Connection;

/// Too lazy to explain... 
/// https://www.bittorrent.org/beps/bep_0003.html#peer-protocol:~:text=protocol%20as%20well.-,peer%20protocol,-BitTorrent%27s%20peer%20protocol


#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Peer {
    pub ip : Ipv4Addr,
    pub port : u16
}

impl Peer {
    pub async fn connect(&self) -> Result<Connection, std::io::Error> {
        Connection::connect(self).await
    }
}