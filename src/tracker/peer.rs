use std::ops::Deref;

use rand::{
    rng,
    RngCore,
};

pub struct Peer {
    
}

pub struct PeerId([u8;20]);

impl PeerId {
    pub fn new() -> Self {
        let mut byte_array: [u8; 20] = [0; 20];

        byte_array[..8].copy_from_slice(b"-OREKOO-");
        rng().fill_bytes(&mut byte_array[8..]);

        Self(byte_array)
    }

    pub fn url_encoded(&self) -> String {
        self.0.iter().map(|x| format!("%{x:02X}")).collect()
    }
}

impl Deref for PeerId {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}