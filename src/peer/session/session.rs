use crate::{
    peer::{
        session::{
            self, Error,
            Event::{self, *},
        },
        Connection, Message,
    },
    torrent,
};

use bytes::Bytes;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub struct Session {
    connection: Connection,
    last_active: Instant,
    torrent_info: Arc<torrent::Info>,
    state: Arc<Mutex<torrent::State>>,
    is_choking: bool,
    is_interested: bool,
    am_choking: bool,
    am_interested: bool,
    bit_field: Option<Bytes>,
}

use tokio::{sync::Mutex, time::timeout};
use Message::*;

impl Session {
    pub fn new(connection : Connection, torrent_info : torrent::Info, state : torrent::State) {
        
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

            let event = self.handle_message(message).await?;

            match event {
                BitFieldUpdated => self.handle_bitfield().await?,
                UnchokedMe => {
                    if self.am_interested {
                        self.request_block().await?
                    }
                }
                PieceRecieved {
                    index,
                    offset,
                    data,
                } => self.handle_piece(index, offset, data).await?,
                Event::KeepAlive => {}
                x => todo!("Unimplemented event recieved : {x:?}"),
            }
        }
    }

    async fn handle_piece(&mut self, index: u32, offsset: u32, data: Bytes) -> Result<(), Error> {
        todo!("Ok ill pick the file, get its guard, and edit it, lol")
    }

    async fn handle_message(&mut self, message: Message) -> Result<session::Event, session::Error> {
        match message {
            Bitfield(x) => {
                self.bit_field = Some(x);
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
            Request { index, offset, length } => {
                todo!("Can't handle requests yet")
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

    fn peer_has_something_i_dont(mine: &[u8], peer: &[u8]) -> bool {
        peer.iter()
            .zip(mine.iter())
            .any(|(peer, mine)| peer & !mine != 0)
    }

    async fn handle_bitfield(&mut self) -> Result<(), Error> {
        let peer_bitfield = self.bit_field.as_ref().unwrap();
        let i_need_piece = {
            let my_bitfield = &self.state.lock().await.bit_field;
            Self::peer_has_something_i_dont(peer_bitfield.as_ref(), &my_bitfield)
        };
        if i_need_piece {
            self.am_interested = true;
            self.connection.send(Message::Interested).await?;
        }
        Ok(())
    }

    async fn request_block(&self) -> Result<(), Error> {
        todo!()
    }
}
