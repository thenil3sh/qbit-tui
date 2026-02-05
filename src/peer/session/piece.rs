use std::collections::{HashSet, VecDeque};

use bytes::{Bytes, BytesMut};
use rand::Fill;

use sha1::{Digest, Sha1};

use crate::{peer::{session::{self, piece}, Message, PeerSession as Session}, torrent::CommitJob};

pub struct Piece {
    index: u32,

    piece_len: u32,
    max_block_len: u32,

    pending: VecDeque<Block>,
    on_fly: HashSet<u32>,
    received: HashSet<u32>,

    buffer: BytesMut,
}

impl Piece {
    pub fn new(index: u32, piece_len: u32) -> Self {
        let mut offset = 0;
        let max_block_len = 16384; // HARD CODED, i'll take care of it

        let mut pending = VecDeque::with_capacity((piece_len / max_block_len + 1) as usize);
        while offset < piece_len {
            if offset + max_block_len > piece_len {
                pending.push_back(Block::new(offset, piece_len - offset));
            } else {
                pending.push_back(Block::new(offset, max_block_len));
            }
            offset += max_block_len;
        }
        let buffer = BytesMut::zeroed(piece_len as usize);
        Self {
            index,
            max_block_len,
            pending,
            piece_len,
            on_fly: HashSet::new(),
            received: HashSet::new(),

            buffer,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    fn total_blocks(&self) -> u32 {
        (self.piece_len + self.max_block_len - 1) / self.max_block_len
    }


    /// Returns a `Some(Message::Request)` of a Piece block\
    /// Returns None up when when pipeline's on-fly capacity is reached
    /// ## Example
    /// ```rs
    /// while let Some(request) = piece.next_block() {
    ///         /* Request here maybe */
    /// }
    /// ```
    pub fn next_block(&mut self) -> Option<Message> {
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
    
    /// Loses ownership of the buffer, also Piece gets moved
    pub fn owned_buffer(self) -> Bytes {
        self.buffer.freeze()
    }

    pub fn can_request_more(&self) -> bool {
        self.on_fly.len() < 4
    }

    pub fn has_pending_req(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.received.len(), self.total_blocks() as usize)
    }

    fn in_flight_count(&self) -> usize {
        self.on_fly.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn update_buffer(&mut self, index: u32, offset: u32, data: &[u8]) -> piece::Result<()> {
        let expected_len = if offset + self.max_block_len > self.piece_len {
            self.piece_len - offset
        } else {
            self.max_block_len
        };

        if expected_len > self.max_block_len
            || self.index != index
            || expected_len != data.len() as u32
        {
            eprintln!("\x1b[37mSelf.index : {}, found : {}\x1b[0m",
                self.index(),
                index
            );

            return Err(Error::BadPiece);
        } else if self.received.contains(&offset) {
            return Err(Error::DuplicateBlock {
                index: self.index,
                block: offset,
            });
        } else if !self.on_fly.contains(&offset) {
            eprintln!("{:?}", self.on_fly);
            return Err(Error::UnexpectedBlock {
                index: self.index,
                block: offset,
            });
        }

        if offset + expected_len > self.buffer.len() as u32 {
            eprintln!("\x1b[31mAccessing offset {}, but length is {}\x1b[0m", offset + expected_len, self.buffer.len());
        }
        
        self.buffer[offset as usize..(offset + expected_len) as usize].copy_from_slice(data);
        self.on_fly.remove(&offset);
        self.received.insert(offset);
        Ok(())
    }

    
    pub fn rebuild_pending(&mut self) {
        self.pending.clear();
        let mut offset = 0;
        while offset < self.piece_len {
            let len = self.max_block_len.min(self.piece_len - offset);
            self.pending.push_back(Block::new(offset, len));
            offset += self.max_block_len;
        }
    }

    pub fn reset(&mut self) {
        self.buffer = BytesMut::with_capacity(self.piece_len as usize);
        self.buffer.fill(0);
        self.on_fly.clear();
        self.pending = VecDeque::with_capacity(self.total_blocks() as usize);
        self.received.clear();
        self.rebuild_pending();
    }


    pub fn verify(&self, pieces : &[u8]) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(&self.buffer);
        let result = hasher.finalize();

        let index = self.index as usize;
        let range = index * 20..index * 20 + 20;
        let pieces = &pieces[range];

        *pieces == *result
    }

    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    pub fn is_complete(&self) -> bool {
        let expected_blocks = self.total_blocks();
        return expected_blocks == self.received.len() as u32 && self.on_fly.is_empty();
    }
}

impl Session {
    /// Forwards downloaded piece to committer, leaving self.current_piece with None
    pub(crate) async fn handle_completed_piece(&mut self) -> session::Result<()> {
        let commit_job = CommitJob::from(
            self.current_piece
                // Self piece is none, from this point
                .take()
                .expect("Tried to take out a None Piece, got smacked"),
        );

        self.commit_tx.send(commit_job).await?;

        if let Some(index) = self.reserve_interesting_piece().await {
            self.current_piece = Some(Piece::new(index, self.torrent_info.piece_len(index)));
            self.pump_requests().await?;
        }
        // Shutup, I'm having a break
        // sleep(Duration::from_millis(1000)).await;

        Ok(())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Received bad piece from peer")]
    BadPiece,

    #[error("Peer send a duplicate block")]
    DuplicateBlock { index: u32, block: u32 },

    #[error("Recieved invalid block from peer")]
    UnexpectedBlock { index: u32, block: u32 },

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
