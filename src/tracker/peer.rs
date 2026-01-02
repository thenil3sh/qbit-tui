use std::{net::Ipv4Addr, ops::Deref};
use rand::{
    rng,
    RngCore,
};
use serde::Deserialize;

///////// Gotta fix it later
#[derive(Copy, Clone, Deserialize, Debug)]
pub struct Peer {
    pub ip : Ipv4Addr,
    pub port : u16
}

pub struct Id([u8;20]);

impl Id {
    /// Idiomatic, peer::Id::new(), lol
    pub fn new() -> Self {
        let mut byte_array: [u8; 20] = [0; 20];

        byte_array[..8].copy_from_slice(b"-OREKOO-");
        rng().fill_bytes(&mut byte_array[8..]);

        Self(byte_array)
    }
    
    /// Since url accepts percentage encoding only, converting each and every character to this encode won't hurt, alternative to this, form::url_encoded exists.
    pub fn url_encoded(&self) -> String {
        self.0.iter().map(|x| format!("%{x:02X}")).collect()
    }
}

impl Deref for Id {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
