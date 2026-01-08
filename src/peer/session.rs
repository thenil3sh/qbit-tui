use crate::{peer::{connection, Connection, Message}, torrent};
use std::{sync::Arc, time::Instant};
use anyhow::{Error, anyhow};
use bytes::Bytes;
use serde::de::Unexpected;

pub struct Session {
    connection : Connection,
    last_active : Instant,
    torrent_info : Arc<torrent::Info>,
    state : Arc<Mutex<torrent::State>>,
    is_choking : bool,
    is_interested : bool,
    am_choking : bool,
    am_interested : bool,
    bit_field : Option<Bytes>
}

use tokio::sync::Mutex;
use Message::*;

impl Session {

    async fn run() {
        todo!()
    }

    async fn handle_message(&mut self, message : Message) -> Result<(), SessionError> {
        match message {
            Bitfield(x) => {
                self.bit_field = Some(x);
                if self.peer_has_something_i_want() {
                    self.am_interested = true;
                    self.connection.send_interested().await?;
                }
            },
            Choke => self.is_choking = true,
            Unchoke => {
                self.is_choking = false;
                if self.am_interested {
                    self.request_block().await?;
                }
            }
            Interested => self.is_interested = true,
            Piece { index, offset, data : _ } => eprintln!("Recv Piece {{ index : {index}, offset : {offset}, }}"),
            KeepAlive => {} // keeps alive, lol, see Self::run()
            NotInterested => self.is_interested = false,
            UnexpectedId(_) => return Err(SessionError::ProtocolViolation)
        }
        Ok(())
    }

    fn peer_has_something_i_want(&self) -> bool {
        todo!("Yet to be implemented : Check on peer, if they have something you need");
    }
    
    fn has_piece(&self) -> bool {
        todo!("Iterate across bitfield, it's prolly got something you want");
        todo!("I'll remove this method, i guess");
    }

    fn handle_bitfield(&mut self, bitfield : Bytes) {
        self.bit_field = Some(bitfield)
        
    }

    async fn request_block(&self) -> Result<(), SessionError>{
        todo!()
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum SessionError {
    #[error("Protocol violation")]
    ProtocolViolation,
    #[error("Session timed out")]
    TimeOut,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}