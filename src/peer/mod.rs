pub mod id;
mod handshake;
mod connection;
mod message;
mod session;

use std::net::Ipv4Addr;

pub use id::PEER_ID as ID;
use serde::Deserialize;
pub use handshake::Handshake;
pub use connection::Connection as Connection;
pub use message::*;
pub use session::Session as PeerSession;
pub(crate) use session::Error as SessionError;
pub(crate) use session::Piece;

use crate::peer::id::Id;

/// Too lazy to explain... 
/// https://www.bittorrent.org/beps/bep_0003.html#peer-protocol:~:text=protocol%20as%20well.-,peer%20protocol,-BitTorrent%27s%20peer%20protocol


#[derive(Copy, Clone, Deserialize, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Peer {
    pub ip : Ipv4Addr,
    pub port : u16,

    #[serde(default)]
    pub id : Option<Id>,
}

impl Peer {
    pub async fn connect(&self) -> Result<Connection, std::io::Error> {
        Connection::connect(*self).await
    }
}