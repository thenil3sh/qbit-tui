use bytes::Bytes;

use crate::peer::Piece;

pub struct Job {
    pub(crate) index : u32,
    pub(crate) bytes : Bytes
}

impl Job {
    fn new(index : u32, bytes : Bytes) -> Self {
        Self { index, bytes }
    }
}

impl From<Piece> for Job {
    fn from(piece: Piece) -> Self {
       Self {
           index : piece.index(),
           bytes : piece.owned_buffer(),
       }
    }
}