use std::net::{IpAddr::V4, SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::peer::{Handshake, Peer};

#[allow(unused)] ///////////////////// For nowww 
pub struct Connection {
    peer: Peer,
    stream: TcpStream,
}

impl Connection {
    pub async fn connect(peer: Peer) -> Result<Self, std::io::Error> {
        let socket_addr = SocketAddr::new(V4(peer.ip), peer.port);
        let stream = TcpStream::connect(socket_addr).await?;

        Ok(Self { peer, stream })
    }

    pub async fn handshake(&mut self, handshake: Handshake) -> Result<(), std::io::Error> {
        self.stream.write_all(handshake.bytes()).await?;

        let mut response_buffer = [0u8; 68];
        self.stream.read_exact(&mut response_buffer).await?;
        Ok(())
    }
}
