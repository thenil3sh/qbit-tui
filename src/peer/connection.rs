use std::net::{IpAddr::V4, SocketAddr};
use bytes::BytesMut;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use bytes::Bytes;

use crate::peer::{Handshake, Message, Peer};

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
    
    pub async fn handle(&mut self) -> io::Result<Message> {
        let length = self.stream.read_u32().await.unwrap();
        
        if length == 0 {
            // TODO() : i'll reset the connection timeout
            return Ok(Message::KeepAlive);
        }
        let id = self.stream.read_u8().await?;
        let payload = Self::scrap_payload(&mut self.stream, length as usize).await?;
        
        Ok(Message::decode(id, payload)?)
    }
    
    async fn scrap_payload(stream : &mut TcpStream, len : usize) -> io::Result<Bytes> {
        let mut buffer = BytesMut::with_capacity(len - 1);
        stream.read_exact(&mut buffer).await?;
        
        Ok(buffer.freeze())
    }
}

#[cfg(test)]
mod tests {
    
}
