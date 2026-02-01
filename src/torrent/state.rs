use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::torrent::{InfoHash, Metadata};
use std::io::{self, Write};
use std::{collections::HashSet, path::PathBuf};
use tokio::fs::{self, File, create_dir_all};

use crate::torrent::{self};

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    downloaded: usize,
    pub(crate) bit_field: Vec<u8>,
    
    #[serde(skip)]
    in_flight: HashSet<u32>,
    num_pieces: u32,
}

impl State {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn load_or_new(torrent: &Metadata) -> Self {
        match Self::try_load(torrent).await {
            Ok(state) => state,
            Err(err) => {
                eprintln!("Loading state failed, starting a fresh one | {err}");
                Self::try_from(torrent).expect("Metadata must be in a valid format")
            }
        }
    }

    async fn try_load(torrent: &Metadata) -> io::Result<Self> {
        let path = Self::path(&torrent.info_hash).await?;
        let mut file = File::open(path).await?;
        Self::from_file(&mut file).await
    }

    async fn from_file(file: &mut File) -> Result<Self, io::Error> {
        let mut vec = Vec::new();
        file.read_to_end(&mut vec).await?;

        Self::from_bytes(vec)
    }

    fn from_bytes<T>(bytes: T) -> Result<Self, io::Error>
    where
        T: AsRef<[u8]>,
    {
        Self::try_from(bytes.as_ref())
    }

    /// Serializes state into CBOR bytes
    fn to_bytes(&self) -> Result<Vec<u8>, io::Error> {
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(self, &mut buffer)
            .map_err(|x| io::Error::new(io::ErrorKind::InvalidData, x))?;
        Ok(buffer)
    }

    async fn save(&self, info_hash: &InfoHash) -> Result<(), io::Error> {
        let path = Self::path(info_hash).await?;
        let tmp = path.with_extension("tmp");

        let bytes = self.to_bytes()?;

        fs::write(&tmp, bytes).await?;
        fs::rename(tmp, path).await?;

        Ok(())
    }

    async fn path(info_hash: &InfoHash) -> io::Result<PathBuf> {
        let path = dirs::data_dir()
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "data directory not found",
            ))?
            .join("qbit")
            .join(info_hash.to_string());

        create_dir_all(&path).await?;
        Ok(path.join("state.cbor"))
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
            Err(x) => Err(io::Error::new(io::ErrorKind::InvalidData, x)),
        };
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, env};

    use serial_test::serial;
    use tokio::fs;

    use crate::torrent::{Metadata, State};

    /// Simulating data directory for tests, it's unsafe, better use it within single threaded environments
    async fn with_temp_dir<F, Fut, T>(f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let old = env::var_os("XDG_DATA_HOME");
        let result;

        // See std::env::set_var and std::env::remove_var's docs
        unsafe {
            std::env::set_var("XDG_DATA_HOME", temp_dir.path());
            result = f().await;
            match old {
                Some(x) => env::set_var("XDG_DATA_HOME", x),
                None => env::remove_var("XDG_DATA_HOME"),
            }
        }
        result
    }
    
    #[tokio::test]
    #[serial]
    async fn inflight_is_not_persisted() {
        with_temp_dir(|| async {
            let metadata = Metadata::fake();
    
            let mut state = State::try_from(&metadata).unwrap();
            state.add_in_flight(2);
            state.mark_piece_complete(1);
    
            state.save(&metadata.info_hash).await.unwrap();
            let loaded = State::load_or_new(&metadata).await;
    
            assert!(!loaded.is_in_flight(2));
            assert!(loaded.have_piece(1));
        }).await;
    }


    #[tokio::test]
    #[serial]
    async fn load_new_metadata_if_not_already_exist() {
        let meta = Metadata::fake();
        let state = with_temp_dir(|| State::load_or_new(&meta)).await;

        assert!(!state.is_complete());
        assert_eq!(state.completed_pieces(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn saving_and_loading_states_are_fine() {
        let metadata = Metadata::fake();

        let mut state = State {
            bit_field: vec![0; 2],
            num_pieces: 16,
            in_flight: HashSet::from([3, 4]),
            ..Default::default()
        };
        state.mark_piece_complete(4);
        state.mark_piece_complete(3);
        let loaded_state = with_temp_dir(|| async {
            state.save(&metadata.info_hash).await.unwrap();

            State::load_or_new(&metadata).await
        })
        .await;

        assert!(loaded_state.have_piece(4));
        assert!(loaded_state.have_piece(3));

        assert_eq!(loaded_state.completed_pieces(), 2);
    }
    
    #[tokio::test]
    #[serial]
    async fn reading_a_corrupted_state() {
        let metadata = Metadata::fake();
        let state = with_temp_dir(|| async {
            let path = State::path(&metadata.info_hash).await.unwrap();
            
            fs::write(path, b"Definately not a valid cbor").await.unwrap();
            
            State::load_or_new(&metadata).await
        }).await;
        
        assert!(!state.is_complete());
        assert_eq!(state.completed_pieces(), 0);
    }

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
        assert!(!state.have_piece(1));
    }

    #[test]
    fn is_complete_works_on_full_bytes() {
        let state = State {
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
        
        state.in_flight.insert(2);
        for _ in 0..20 {
            state.mark_piece_complete(2);
        }

        assert!(state.downloaded <= state.num_pieces as usize);
        assert_eq!(state.completed_pieces(), 1);
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
