use std::{
    cmp::max,
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use tokio::sync::Mutex;

use sha1::{Digest, Sha1};

use crate::{
    peer::{Message, SessionError, session::piece},
    torrent::Info,
};

pub struct Piece {
    index: u32,

    piece_len: u32,
    max_block_len: u32,

    pending: VecDeque<Block>,
    on_fly: HashSet<u32>,
    received: HashSet<u32>,

    buffer: Vec<u8>,
}

impl Piece {
    fn new(index: u32, piece_len: u32) -> Self {
        let mut offset = 0;
        let max_block_len = 16384;
        let mut pending = VecDeque::with_capacity((piece_len / max_block_len + 1) as usize);
        while offset < piece_len {
            if offset + max_block_len > piece_len {
                pending.push_back(Block::new(offset, piece_len - offset));
            } else {
                pending.push_back(Block::new(offset, max_block_len));
            }
            offset += max_block_len;
        }
        Self {
            index,
            max_block_len,
            pending,
            piece_len,
            on_fly: HashSet::new(),
            received: HashSet::new(),

            buffer: vec![0; piece_len as usize],
        }
    }

    fn total_blocks(&self) -> u32 {
        (self.piece_len + self.max_block_len - 1) / self.max_block_len
    }

    fn next_block(&mut self) -> Option<Message> {
        if let Some(Block { offset, length }) = self.pending.pop_front() {
            self.on_fly.insert(offset);
            return Some(Message::Request {
                index: self.index,
                offset,
                length,
            });
        }
        None
    }

    fn has_pending_req(&self) -> bool {
        !self.pending.is_empty()
    }

    fn progress(&self) -> (usize, usize) {
        (self.received.len(), self.total_blocks() as usize)
    }

    fn in_flight_count(&self) -> usize {
        self.on_fly.len()
    }

    fn pending_count(&self) -> usize {
        self.pending.len()
    }

    fn update_buffer(&mut self, index: u32, offset: u32, data: &[u8]) -> piece::Result<()> {
        let expected_len = if offset + self.max_block_len > self.piece_len {
            self.piece_len - offset
        } else {
            self.max_block_len
        };

        if expected_len > self.max_block_len
            || self.index != index
            || expected_len != data.len() as u32
        {
            return Err(Error::BadPiece);
        } else if self.received.contains(&offset) {
            return Err(Error::DuplicateBlock);
        } else if !self.on_fly.contains(&offset) {
            return Err(Error::UnexpectedBlock);
        }
        self.buffer[offset as usize..(offset + expected_len) as usize].copy_from_slice(data);
        self.on_fly.remove(&offset);
        self.received.insert(offset);
        Ok(())
    }

    fn rebuild_pending(&mut self) {
        self.pending.clear();
        let mut offset = 0;
        while offset < self.piece_len {
            let len = self.max_block_len.min(self.piece_len - offset);
            self.pending.push_back(Block::new(offset, len));
            offset += self.max_block_len;
        }
    }

    fn reset(&mut self) {
        self.buffer = vec![0u8; self.piece_len as usize];
        self.on_fly.clear();
        self.pending = VecDeque::with_capacity(self.total_blocks() as usize);
        self.received.clear();
        self.rebuild_pending();
    }

    fn verify(&self, expected_hash: &[u8; 20]) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(&self.buffer);
        let result = hasher.finalize();

        let start = (self.index * 20) as usize;
        let end = start + 20;
        return *expected_hash == *result;
    }

    fn data(&self) -> &[u8] {
        &self.buffer
    }

    fn is_complete(&self) -> bool {
        let expected_blocks = self.total_blocks();
        return expected_blocks == self.received.len() as u32 && self.on_fly.is_empty();
    }

    fn piece_length_from() {}
}

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("received bad piece from peer")]
    BadPiece,
    #[error("Peer send a duplicate block")]
    DuplicateBlock,
    #[error("Invalid piece recieved from peer")]
    UnexpectedBlock,
    #[error("Recieved block with invalid block length")]
    InvalidBlockLength,
    #[error("Recieved piece with invalid index")]
    InvalidPieceIndex,
    #[error("Hash check failed")]
    HashMismatch,
}

struct Block {
    offset: u32,
    length: u32,
}

impl Block {
    fn new(offset: u32, length: u32) -> Self {
        Self { offset, length }
    }
}
