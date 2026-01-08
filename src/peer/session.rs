use crate::peer::{Connection, Message, connection};
use std::time::{Instant};
use anyhow::{Error, anyhow};
use bytes::Bytes;
use serde::de::Unexpected;

pub struct Session {
    connection : Connection,
    last_active : Instant,

    is_choking : bool,
    is_interested : bool,

    am_choking : bool,
    am_interested : bool,

    bit_field : Option<Bytes>
}

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
        todo!()
    }

    fn handle_bitfield(&mut self, bitfield : Bytes) {
        self.bit_field = Some(bitfield)
        
    }

    async fn request_block(&self) -> Result<(), SessionError>{
        todo!()
    }
}


enum SessionError {
    ProtocolViolation
}

impl From<Connection> for Session {
    fn from(connection: Connection) -> Self {
        Self {
            connection,
            last_active : Instant::now(),
            is_choking : true,
            is_interested : false,
            am_choking : true,
            am_interested : false,

            bit_field : None
        }
    }
}