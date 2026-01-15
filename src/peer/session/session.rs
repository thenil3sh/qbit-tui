use crate::{
    peer::{
        Connection, Message,
        session::{
            self, Error,
            Event::{self, *},
        },
    },
    torrent,
};

use bytes::Bytes;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

pub struct Session {
    connection: Connection,
    last_active: Instant,
    torrent_info: Arc<torrent::Metadata>,
    current_piece: Option<u32>,
    current_offset: u32,
    state: Arc<Mutex<torrent::State>>,
    is_choking: bool,
    is_interested: bool,
    am_choking: bool,
    request_queue: VecDeque<Message>,
    am_interested: bool,
    bit_field: Option<Vec<u8>>,
}

use Message::*;
use tokio::{io, sync::Mutex, time::timeout};

impl Session {
    pub fn new(
        connection: Connection,
        torrent_info: Arc<torrent::Metadata>,
        state: Arc<Mutex<torrent::State>>,
    ) -> Self {
        Self {
            connection,
            last_active: Instant::now(),
            torrent_info,
            state,
            current_piece: None,
            current_offset: 0,
            is_choking: true,
            is_interested: false,
            request_queue: VecDeque::new(),
            am_choking: true,
            am_interested: false,
            bit_field: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            let time_left = Duration::from_secs(120) - (Instant::now() - self.last_active);
            let timeout = timeout(time_left, self.connection.read_message()).await;

            let message = if timeout.is_err() {
                return Err(Error::TimeOut);
            } else {
                timeout.unwrap()?
            };
            self.last_active = Instant::now();
            eprintln!("Got message : {message:?}");
            let event = self.handle_message(message).await?;

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
                x => eprintln!("Unimplemented event recieved : {x:?}"),
            }
        }
    }

    async fn handle_choked_me(&mut self) -> io::Result<()> {
        self.is_choking = true;
        if let Some(piece) = self.current_piece {
            self.state.lock().await.remove_in_flight(piece);
        }
        Ok(())
    }

    async fn handle_piece(&mut self, index: u32, offset: u32, data: Bytes) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_have(&mut self, index: u32) -> Result<(), Error> {
        let interesting_piece = self.reserve_interesting_piece().await;

        if !self.is_choking && interesting_piece.is_some() {
            self.connection
                .send(Message::Request {
                    index,
                    offset: 0,
                    length: 16384,
                })
                .await?;
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
            Piece {
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
    /// Else it'll 
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

    async fn handle_unchoke(&mut self) -> session::Result<()> {
        if self.am_interested
            && let Some(index) = self.reserve_interesting_piece().await
        {
            self.state.lock().await.add_in_flight(index);
            self.connection
                .send(Message::Request {
                    index,
                    offset: 0,
                    length: 16384,
                })
                .await?;
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

    async fn request_block(&self) -> Result<(), Error> {
        todo!()
    }
}
