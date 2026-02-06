use bytes::Bytes;
use bytes::BytesMut;
use std::net::{IpAddr::V4, SocketAddr};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::peer::{session, Handshake, Message, Peer};

#[allow(unused)] ///////////////////// For nowww
#[derive(Debug)]
pub struct Connection {
    pub(crate) peer: Peer,
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

    pub async fn read_message(&mut self) -> Result<Message, io::Error> {
        let length = self.stream.read_u32().await?;

        if length == 0 {
            return Ok(Message::KeepAlive);
        }
        let id = self.stream.read_u8().await?;
        let payload = Self::scrap_payload(&mut self.stream, length as usize).await?;
        let message = Message::decode(id, payload)?;
        eprintln!("\x1b[30mSESSION {} | Recieved : {:?}\x1b[0m", self.peer.ip, message);
        Ok(message)
    }

    async fn scrap_payload(stream: &mut TcpStream, len: usize) -> io::Result<Bytes> {
        let mut buffer = BytesMut::with_capacity(len - 1);
        buffer.resize(len - 1, 0);
        stream.read_exact(&mut buffer).await?;

        Ok(buffer.freeze())
    }

    /// Writes the encoded message to the TCP stream
    pub(crate) async fn send(&mut self, message : Message) -> Result<(), session::Error> {
        self.stream.write_all(&message.encode()).await?;
        println!("SESSION {} | Recieved : {:?}", self.peer.ip, message);
        Ok(())
    }
}


#[cfg(test)]
mod tests {}