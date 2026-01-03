use crate::{peer::{self}, torrent::{self, InfoHash}};


pub struct Handshake([u8;68]);

impl Handshake {
    
    /// Gives out raw handshaking template leaving info_hash field indices free
    fn raw() -> [u8; 68] {
        let mut buffer = [0u8; 68];
        buffer[0] = 19;
        buffer[1..20].copy_from_slice(b"BitTorrent protocol");
        buffer[48..].copy_from_slice(&peer::ID);
        
        buffer
    }

    pub fn new(info_hash : &InfoHash) -> Self {
        let mut buffer = Self::raw();
        buffer[28..48].copy_from_slice(info_hash.as_ref());

        Self(buffer)
    }
    
}

impl From<torrent::Metadata> for Handshake {
    fn from(metadata: torrent::Metadata) -> Self {
        Handshake::new(&metadata.info_hash)
    }
}

impl From<&InfoHash> for Handshake {
    fn from(info_hash: &InfoHash) -> Self {
        Handshake::new(info_hash)
    }
}

impl AsRef<[u8;68]> for Handshake {
    fn as_ref(&self) -> &[u8;68] {
        &self.0
    }
}

impl AsMut<[u8;68]> for Handshake {
    fn as_mut(&mut self) -> &mut [u8;68] {
        &mut self.0
    }
}

