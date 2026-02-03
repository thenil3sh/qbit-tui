use crate::{
    peer::{
        Connection, Message,
        session::{
            self, Error,
            Event::{self, *},
            Piece,
        },
    },
    torrent::{self, CommitEvent, Committer, commit},
};

use bytes::Bytes;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct Session {
    commit_rx: broadcast::Receiver<commit::Event>,
    connection: Connection,
    last_active: Instant,
    torrent_info: Arc<torrent::Info>,
    current_piece: Option<Piece>,
    state: Arc<Mutex<torrent::State>>,
    is_choking: bool,
    is_interested: bool,
    am_choking: bool,
    am_interested: bool,
    bit_field: Option<Vec<u8>>,
}

use Message::*;
use tokio::{
    io,
    sync::{Mutex, broadcast, mpsc},
    time::{sleep, timeout},
};

impl Session {
    pub fn new(
        connection: Connection,
        torrent_info: Arc<torrent::Info>,
        state: Arc<Mutex<torrent::State>>,
        commit_rx: broadcast::Receiver<commit::Event>,
    ) -> Self {
        Self {
            commit_rx,
            connection,
            last_active: Instant::now(),
            torrent_info,
            state,
            current_piece: None,
            is_choking: true,
            is_interested: false,
            am_choking: true,
            am_interested: false,
            bit_field: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
                message = self.connection.read_message() => {
                    let message = message?;
                    eprintln!("Got message : {message:?}");
                    let event = self.handle_message(message).await?;
                    self.handle_event(event).await?;
                }
                commit = self.commit_rx.recv() => {
                    self.handle_commit_event(commit.unwrap()).await?;
                }
                _ = tokio::time::sleep(Duration::from_secs(120)) => {
                    return Err(Error::TimeOut)
                }
            }
        }
    }

    async fn handle_commit_event(&mut self, event: commit::Event) -> session::Result<()> {
        match event {
            CommitEvent::PieceCommit(index) => self.connection.send(Message::Have(index)).await?,
            CommitEvent::FailedCommit => todo!("Fokin failed?")
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: session::Event) -> session::Result<()> {
        match event {
            BitFieldUpdated => self.handle_bitfield().await?,
            UnchokedMe => self.handle_unchoke().await?,
            Event::Have(x) => self.handle_have(x).await?,
            PieceRecieved {
                index,
                offset,
                data,
            } => self.handle_piece(index, offset, data).await?,
            Event::ChokedMe => self.handle_choked_me().await?,
            Event::KeepAlive => {}
            x => eprintln!("\x1b[034mUnimplemented event recieved : {x:?}\x1b[0m"),
        }
        Ok(())
    }

    async fn handle_choked_me(&mut self) -> io::Result<()> {
        self.is_choking = true;
        if let Some(piece) = self.current_piece.as_ref() {
            self.state.lock().await.remove_in_flight(piece.index());
            // todo!("Need some rework here")
        }
        Ok(())
    }

    async fn handle_piece(&mut self, index: u32, offset: u32, data: Bytes) -> Result<(), Error> {
        if self.current_piece.is_none() {
            return Err(Error::ProtocolViolation);
        }
        let piece = self.current_piece.as_mut().unwrap();
        piece.update_buffer(index, offset, data.as_ref())?;
        if piece.is_complete() {
            if piece.verify(&self.torrent_info.pieces) {
                eprintln!(
                    "\x1b[35m\x1b[1mDownloaded piece : {index} + VERIFIED CHECKSUM!!!\x1b[0m"
                );
            }
            self.current_piece = None;
            todo!("Sender fucking needs a sender bro");
        }
        self.pump_requests().await?;
        Ok(())
    }

    /// Have is the piece a peer wants to tell that they have.
    /// It's possible to be interested in that very piece, moreover, it's worth updating their bitfield to keep track of their pieces.
    async fn handle_have(&mut self, index: u32) -> Result<(), Error> {
        self.update_bitfield(index)?;

        if !self.is_choking && self.current_piece.is_none() {
            if let Some(index) = self.reserve_interesting_piece().await {
                self.current_piece = Some(Piece::new(index, self.torrent_info.piece_len(index)));
                self.pump_requests().await?;
            }
        }
        Ok(())
    }

    fn update_bitfield(&mut self, index: u32) -> session::Result<()> {
        let piece = index as usize;
        let byte = piece / 8;
        if byte >= self.bit_field.as_ref().unwrap().len() {
            return Err(Error::BadRequest);
        }
        let bit = piece % 8;

        let mask = 1 << (7 - bit);

        self.bit_field.as_mut().unwrap()[byte] |= mask;
        Ok(())
    }

    async fn handle_message(&mut self, message: Message) -> Result<session::Event, session::Error> {
        match message {
            Bitfield(x) => {
                self.bit_field = Some(x.into());
                Ok(Event::BitFieldUpdated)
            }
            Choke => {
                self.is_choking = true;
                Ok(Event::ChokedMe)
            }
            Unchoke => {
                self.is_choking = false;
                Ok(Event::UnchokedMe)
            }
            Request {
                index,
                offset,
                length,
            } => {
                todo!("Can't handle requests yet")
            }
            Message::Have(x) => {
                if self.bit_field.is_none() {
                    let len = self.state.lock().await.num_pieces() / 8;
                    self.bit_field = Some(vec![0u8; len as usize]);
                }
                self.update_bitfield(x)?;
                Ok(Event::Have(x))
            }
            Interested => {
                self.is_interested = true;
                Ok(Event::PeerInterested)
            }
            Message::Piece {
                index,
                offset,
                data,
            } => Ok(Event::PieceRecieved {
                index,
                offset,
                data,
            }),
            Message::KeepAlive => Ok(Event::KeepAlive), // keeps alive, lol, see Self::run()
            NotInterested => {
                self.is_interested = false;
                Ok(Event::PeerNotInterested)
            }
            UnexpectedId(_) => return Err(Error::ProtocolViolation),
        }
    }

    /// Looks for a piece in peer's bitfield, if there's anything interesting, it'll reserve it, then return the index to user
    /// Else it responds with None
    async fn reserve_interesting_piece(&self) -> Option<u32> {
        let mut state = self.state.lock().await;
        for (byte_idx, (peer, mine)) in self
            .bit_field
            .as_ref()
            .unwrap()
            .iter()
            .zip(state.bit_field.iter())
            .enumerate()
        {
            let difference = peer & !mine;
            if difference == 0 {
                continue;
            }
            for bit in 0..8 {
                let mask = 1 << (7 - bit);
                if difference & mask == 0 {
                    continue;
                }
                let piece = (byte_idx * 8 + bit) as u32;
                if piece >= state.num_pieces() {
                    return None;
                }
                if state.is_in_flight(piece) {
                    continue;
                }

                state.add_in_flight(piece);
                return Some(piece);
            }
        }
        None
    }

    async fn should_be_interested(&self) -> bool {
        let my_state = self.state.lock().await;
        my_state
            .bit_field
            .iter()
            .zip(self.bit_field.as_ref().unwrap().iter())
            .any(|(mine, peer)| !mine & peer != 0)
    }

    /// When unchoked, I'm ready to request piece from peer
    /// Because each peer is assigned a piece here, I'll pipeline the block requests
    async fn handle_unchoke(&mut self) -> session::Result<()> {
        if self.am_interested
            && let Some(index) = self.reserve_interesting_piece().await
        {
            let piece = Piece::new(index, self.torrent_info.piece_len(index));
            self.current_piece = Some(piece);
            self.state.lock().await.add_in_flight(index);
            self.pump_requests().await?;
        }
        Ok(())
    }

    /// Repeatedly places piece block requests in pipeline, upto Piece's on-fly capacity
    /// So let's say if Piece has capacity of handling 4 blocks on-fly, only 4 blocks will be asked at a time
    async fn pump_requests(&mut self) -> session::Result<()> {
        if self.is_choking || !self.am_interested {
            return Ok(());
        }

        let Some(ref mut current_piece) = self.current_piece else {
            return Ok(());
        };

        while current_piece.can_request_more() {
            let Some(piece_request) = current_piece.next_block() else {
                break;
            };
            eprintln!("{piece_request:?} SENT");
            self.connection.send(piece_request).await?;
        }
        Ok(())
    }

    async fn handle_bitfield(&mut self) -> Result<(), Error> {
        if self.should_be_interested().await {
            self.am_interested = true;
            self.connection.send(Message::Interested).await?;
        }
        Ok(())
    }
}
