use bytes::BytesMut;

pub struct Bitfield {
    bitfield: BytesMut,
    len: usize,
}

impl Bitfield {
    pub fn new(total_pieces: usize) -> Self {
        let bitfield_size = (total_pieces as f64 / 8.0).ceil() as usize;
        let mut bitfield = BytesMut::with_capacity(bitfield_size);
        bitfield.resize(bitfield_size, 0);

        Self {
            bitfield,
            len: total_pieces,
        }
    }

    fn byte_len(&self) -> usize {
        (self.len + 7) / 8
    }

    /// Returns total number of indices bitfield is keeping track of
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Marks a piece in bitfield as complete,
    /// returns whether piece was already marked as completed
    pub fn update(&mut self, index: u32) -> Result<bool> {
        if index as usize >= self.len {
            return Err(Error::IndexOutOfBound);
        }
        let piece = index as usize;
        let byte = piece / 8;
        let bit = piece % 8;

        let mask = 1 << (7 - bit);

        let was_complete = self.bitfield[byte] & mask != 0;
        if !was_complete {
            self.bitfield[byte] |= mask;
        }
        Ok(was_complete)
    }

    /// Returns whether current piece has been marked in bitfield.
    pub fn has(&self, index: usize) -> Result<bool> {
        if index >= self.len() {
            return Err(Error::IndexOutOfBound)
        }
        let byte_index = index as usize / 8;
        let to_shift = index as usize % 8;

        let cmp_byte = 128 >> to_shift;

        Ok(self.bitfield[byte_index] & cmp_byte != 0)
    }

    /// Returns underlying bitfield as slice
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
    
    
    pub fn has_any(&self, other : &Bitfield) -> bool {
        debug_assert_eq!(self.len(), other.len());
        
        for (this, that) in self.bitfield.iter().zip(other.bitfield.iter()) {
            let difference = !this & that;
            if difference != 0 {
                return true;
            }
        }
        false
    }

    /// Copies the byte slice update the bitfield
    /// 
    /// ## Error
    /// This method returns with an [`Error::InvalidLength`] if the two slices have different lengths.
    ///
    /// # Note
    /// This method only compares the bitfield's byte array's length with the [`self.bitfield`],
    /// which is again a byte array.
    /// So, even if peer's bitfield have some extra pieces (more likely malformed), their extra pieces will be ignored, and if they have less pieces, 
    /// the method will return with an error.
    /// To understand this -
    /// - Lets say we have 10 piece torrent
    /// - That means, bitfield is `[0b00000000, 0b00000000]`;
    /// - Now, we may want to update it with peer's bitfield, which is `[0b01010101, 0b00100101]`
    /// - Clearly, peer's bitfield is a bit malformed, but it's still passing comparison of lengths, that is `2`
    /// - In this case, peer's bitfield will be considered worthy of updation, and extra bits will be marked as zero.
    /// - Thus we have updated bitfield `[0b01010101, 0b00000000]` (which was expected)
    pub fn update_from_peer<T>(&mut self, bytes: T) -> Result<()>
    where T : AsRef<[u8]>
    {
        let bytes = bytes.as_ref();
        let expected = self.byte_len();
        if bytes.len() != self.byte_len() {
            return Err(Error::InvalidLength {
                expected,
                got: bytes.len(),
            });
        }

        self.bitfield.copy_from_slice(bytes);
        self.clear_unused_trail_units();

        Ok(())
    }

    /// Fills the extra unused (if any) bits of bitfield with zeroes
    ///
    /// ## Panics
    /// Panics when bitfield's length is `0`, as there is no last element to work with
    pub fn clear_unused_trail_units(&mut self) {
        
        let extra_bits = self.len() % 8;
        if self.len() % 8 == 0 {
            return;
        }
        let mask = 0xff << (8 - extra_bits);
        *self.bitfield.last_mut().expect("Bitfield's length is zero") &= mask;
    }
    
    // I may need these for seeding
    // pub fn iter_set(&self) -> impl Iterator<Item = usize> + '_
    // pub fn iter_unset(&self) -> impl Iterator<Item = usize> + '_
}

impl AsRef<[u8]> for Bitfield {
    fn as_ref(&self) -> &[u8] {
        &self.bitfield
    }
}

pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("bitfield index specified is out of bounds")]
    IndexOutOfBound,
    #[error("Unexpected length of bitfield")]
    InvalidLength { expected: usize, got: usize },
}
