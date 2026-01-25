use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, BufReader};

use crate::torrent::{Info, InfoHash, Metadata};
use std::io;
use std::{collections::HashSet, path::PathBuf};
use tokio::fs::{self, create_dir, create_dir_all};

use crate::torrent::{self};

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    downloaded: usize,
    pub(crate) bit_field: Vec<u8>,
    in_flight: HashSet<u32>,
    num_pieces: u32,
}

impl State {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn load_or_new(torrent : &Metadata) -> Self {
        match Self::path(&torrent.info_hash).await {
            Ok(x) => {
                let file = fs::OpenOptions::new().read(true).open(x).await;
                if let Ok(x) = ;

                Self::new()
            }
            Err(_) => Self::try_from(torrent).unwrap()
        }
    }

    async fn from_file<T>(info_hash : &InfoHash) -> Result<Self, io::Error> 
    {
        let path = Self::path(info_hash).await?;
        let mut file = fs::OpenOptions::new().read(true).open(path).await?;
        let mut vec = vec![0u8;file.metadata().await?.len() as usize];
        file.read_exact(&mut vec);
        
        vec.as_ref().try_into()
    }

    async fn path(info_hash: &InfoHash) -> io::Result<PathBuf> {
        let path = dirs::data_dir().expect("Failed to locate data directory")
            .join(info_hash.to_string());
        create_dir_all(&path).await?;
        let mut path = path.join("state");
        path.set_extension("cbor");
        fs::OpenOptions::new()
            .create(true)
            .open(path.clone().join("state"));
        Ok(path)
    }

    pub(crate) fn remove_in_flight(&mut self, piece: u32) {
        self.in_flight.remove(&piece);
    }

    pub(crate) fn add_in_flight(&mut self, piece: u32) {
        self.in_flight.insert(piece);
    }

    pub fn is_in_flight(&self, piece: u32) -> bool {
        self.in_flight.contains(&piece)
    }

    pub(crate) fn mark_piece_complete(&mut self, index: u32) {
        let piece = index as usize;
        let byte = piece / 8;
        let bit = piece % 8;

        let mask = 1 << (7 - bit);

        let was_complete = self.bit_field[byte] & mask != 0;
        if !was_complete {
            self.bit_field[byte] |= mask;

            if self.in_flight.remove(&index) {
                self.downloaded += 1;
            }
        } else {
            self.in_flight.remove(&index);
        }
    }

    fn have_piece(&self, index: u32) -> bool {
        let byte_index = index as usize / 8;
        let to_shift = index as usize % 8;

        let cmp_byte = 128 >> to_shift;

        self.bit_field[byte_index] & cmp_byte != 0
    }

    pub fn num_pieces(&self) -> u32 {
        return self.num_pieces;
    }

    fn completed_pieces(&self) -> usize {
        return self.downloaded;
    }

    fn is_complete(&self) -> bool {
        let index = self.num_pieces / 8;
        let remainder = self.num_pieces % 8;

        if self.bit_field[..index as usize].iter().any(|x| *x != 0xFF) {
            return false;
        }
        if remainder == 0 {
            return true;
        }
        let mask = 0xFF << (8 - remainder);
        return *self.bit_field.last().unwrap() & mask == mask;
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
            metadata.info.pieces.len() / 20
        };
        let bitfield_size = (num_pieces as f64 / 8.0).ceil() as usize;
        let bit_field = vec![0u8; bitfield_size];

        Ok(Self {
            downloaded,
            in_flight,
            num_pieces: num_pieces as u32,
            bit_field,
        })
    }
}

impl TryFrom<&[u8]> for State {
    type Error = io::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        return match ciborium::from_reader(value) {
            Ok(x) => Ok(x),
            Err(ciborium::de::Error::Io(x)) => Err(x),
            _ => panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::torrent::State;

    #[test]
    fn mark_piece_complete_works_fine() {
        let mut state = State {
            bit_field: vec![0; 2], // 16 pieces
            num_pieces: 16,
            ..Default::default()
        };

        state.mark_piece_complete(2);
        assert_eq!(state.bit_field, [0b0010_0000, 0b0000_0000]);

        state.mark_piece_complete(1);
        assert_eq!(state.bit_field, [0b0110_0000, 0b0000_0000]);

        state.mark_piece_complete(8);
        assert_eq!(state.bit_field, [0b0110_0000, 0b1000_0000]);

        state.mark_piece_complete(13);
        assert_eq!(state.bit_field, [0b0110_0000, 0b1000_0100]);
    }

    #[test]
    fn have_pieces_works_well_with_marked_pieces() {
        let mut state = State {
            bit_field: vec![0; 2],
            num_pieces: 16,
            ..Default::default()
        };

        state.mark_piece_complete(3);
        assert!(state.have_piece(3));

        state.mark_piece_complete(15);
        assert!(state.have_piece(15));

        state.mark_piece_complete(0);
        assert!(state.have_piece(0));

        assert!(!state.have_piece(7));
    }

    #[test]
    fn is_complete_works_on_full_bytes() {
        let mut state = State {
            bit_field: vec![0b1111_1111, 0b1111_1111],
            num_pieces: 16,
            ..Default::default()
        };

        assert!(state.is_complete());
    }

    #[test]
    fn is_complete_works_on_partially_filled_bitfield() {
        let state = State {
            bit_field: vec![0b1111_0111, 0b1111_101],
            num_pieces: 13,
            ..Default::default()
        };

        assert!(!state.is_complete());
    }

    #[test]
    fn is_complete_works_with_marker_helper() {
        let mut state = State {
            bit_field: vec![0b1111_0111, 0b1110_0010],
            num_pieces: 13,
            ..Default::default()
        };

        assert!(!state.is_complete());

        state.mark_piece_complete(4);
        assert!(!state.is_complete());

        state.mark_piece_complete(11);
        assert!(!state.is_complete());

        state.mark_piece_complete(12);
        assert!(state.is_complete());
    }

    #[test]
    fn downloaded_items_are_always_less_than_or_equal_to_num_pieces() {
        let mut state = State {
            bit_field: vec![0b0000_0000],
            num_pieces: 14,
            ..Default::default()
        };

        for i in 0..20 {
            state.mark_piece_complete(2);
        }

        assert!(state.downloaded <= state.num_pieces as usize);
    }

    #[test]
    #[should_panic]
    fn marking_unrelated_piece_as_complete() {
        let mut state = State {
            bit_field: vec![0],
            num_pieces: 8,
            ..Default::default()
        };
        state.mark_piece_complete(99);
    }

    #[test]
    #[should_panic]
    fn looking_for_piece_that_never_will_exist() {
        let state = State {
            bit_field: vec![0],
            num_pieces: 8,
            ..Default::default()
        };
        state.have_piece(10);
    }

    #[test]
    fn marking_same_piece_twice_does_not_increment_downloaded() {
        let mut state = State {
            bit_field: vec![0],
            num_pieces: 8,
            ..Default::default()
        };

        state.mark_piece_complete(3);
        let first = state.completed_pieces();

        state.mark_piece_complete(3);
        let second = state.completed_pieces();

        assert_eq!(first, second);
    }
}
