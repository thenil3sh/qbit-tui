use crate::torrent::Metadata;
use std::{collections::HashSet, hash::Hash};

use bytes::Bytes;
use sha1::digest::typenum::bit;

use crate::torrent::{self, metadata};

#[derive(Default)]
pub struct State {
    downloaded: usize,

    pub(crate) bit_field: Vec<u8>,
    in_flight: HashSet<u32>,
    num_pieces: usize,
}

impl State {
    fn mark_piece_complete(&mut self, index : u32) {
        let index = index as usize / 8;
        
    }

    fn have_piece(&self, index: u32) -> bool {
        let bit_index = index as usize / 8;
        let to_shift = index as usize % 8;

        let i = if to_shift == 0 { bit_index } else { bit_index + 1 };
        let cmp_byte = 128 >> to_shift;
        
        self.bit_field[i] & cmp_byte != 0
    }

    fn num_pieces(&self) -> usize {
        return self.num_pieces;
    }

    fn completed_pieces(&self) -> usize {
        return self.downloaded;
    }

    fn is_complete(&self) -> bool {
        let index = self.num_pieces / 8;
        let remainder = self.num_pieces % 8;

        if self.bit_field[..index].iter().any(|x| *x != 0xFF) {
            return false;
        }
        if remainder == 0 {
            return true;
        }
        let mask = 0xFF << (8 - remainder);
        return *self.bit_field.last().unwrap() & mask == mask
    }
}

impl TryFrom<&Metadata> for State {
    type Error = torrent::Error;
    fn try_from(metadata: &Metadata) -> Result<Self, Self::Error> {
        let downloaded = 0;
        let in_flight = HashSet::new();
        let num_pieces = if metadata.info.pieces.len() % 20 != 0 {
            return Err(torrent::Error::InvalidTorrent);
        } else {
            metadata.info.piece_length as usize / 20
        };
        let bitfield_size = (num_pieces as f64 / 8.0).ceil() as usize;
        let bit_field = vec![0u8; bitfield_size];

        Ok(Self {
            downloaded,
            in_flight,
            num_pieces,
            bit_field,
        })
    }
}
