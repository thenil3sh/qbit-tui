pub mod id;
mod handshake;

use std::net::Ipv4Addr;

pub use id::PEER_ID as ID;
use serde::Deserialize;
use handshake::Handshake;



///////// Gotta fix it later
#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Peer {
    pub ip : Ipv4Addr,
    pub port : u16
}