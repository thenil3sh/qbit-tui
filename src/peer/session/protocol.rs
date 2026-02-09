use std::io::{self};

use bytes::Bytes;

use crate::peer::{
    self, Message, PeerSession as Session, Piece, SessionError as Error,
    session::{self, Event},
};

impl Session {
    pub(crate) async fn handle_event(
        &mut self,
        event: peer::session::Event,
    ) -> session::Result<()> {
        match event {
            Event::BitFieldUpdated => self.handle_bitfield().await?,
            Event::UnchokedMe => self.handle_unchoke().await?,
            Event::Have(x) => self.handle_have(x).await?,
            Event::PieceRecieved {
                index,
                offset,
                data,
            } => self.handle_piece(index, offset, data).await?,
            Event::PeerInterested => {
                // Yea event driven unchoke
                self.connection.send(Message::Unchoke).await?;
                eprintln!("SESSION : {} | SENT Unchoke", self.connection.peer.ip);
            }
            Event::ChokedMe => self.handle_choked_me().await?,
            Event::KeepAlive => {}
            Event::PieceRequested {
                index,
                offset,
                length,
            } => {
                // self.handle_piece_request(index, offset, length).await?;
                panic!("SENT A FUCKING PIECE");
            },
            Event::Ignore => {
                eprintln!("\n\n\n\nDUH\n\n\n\n");
            }
            x => eprintln!("\x1b[034mUnimplemented event recieved : {x:?}\x1b[0m"),
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
    
    pub(crate) async fn send_bitfield(&mut self) -> session::Result<()> {
        let message = Message::Bitfield(self.state.lock().await.bit_field.clone().into());
        self.connection.send(message).await?;
        Ok(())
    }

    /// When unchoked, I'm ready to request piece from peer
    /// Because each peer is assigned a piece here, I'll pipeline the block requests
    async fn handle_unchoke(&mut self) -> session::Result<()> {
        if self.am_interested
            && let Some(index) = self.reserve_interesting_piece().await
        {
            let piece = Piece::new(index, self.torrent_info.piece_len(index));
            self.current_piece = Some(piece);
            self.pump_requests().await?;
        }
        Ok(())
    }

    /// Have is the piece a peer wants to tell that they have.
    /// It's possible to be interested in that very piece, moreover, it's worth updating their bitfield to keep track of their pieces.
    ///
    /// # Error
    /// Error may occur in the following cases
    /// - When index is greater than bitfield's index
    /// - Pipelining request to TcpStream
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

    // todo!();
    // async fn read_block(&mut self, index: u32, offset: u32, length: u32) -> io::Result<Bytes> {
    //     let length = length as usize;
    //     let path = self.torrent_info.file_path();
    //     let mut file = fs::OpenOptions::new().read(true).open(path).await?;

    //     let absolute_index = index * self.torrent_info.piece_length + offset;

    //     file.seek(SeekFrom::Start(absolute_index as u64)).await?;

    //     let mut buffer = BytesMut::with_capacity(length as usize);
    //     buffer.resize(length as usize, 0);

    //     file.read_exact(&mut buffer).await?;

    //     Ok(buffer.freeze())
    // }

    // async fn handle_piece_request(
    //     &mut self,
    //     index: u32,
    //     offset: u32,
    //     length: u32,
    // ) -> session::Result<()> {
    //     if self.is_valid_block(index, offset, length).await {
    //         let data = self.read_block(index, offset, length).await?;
    //         self.connection
    //             .send(Message::Piece {
    //                 index,
    //                 offset,
    //                 data,
    //             })
    //             .await?;
    //     } else {
    //         // I'll see what to do
    //     }
    //     Ok(())
    // }

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
        if piece.is_complete() && piece.verify(&self.torrent_info.pieces) {
            eprintln!("\x1b[35m\x1b[1mDownloaded piece : {index} + VERIFIED CHECKSUM!!!\x1b[0m");
            self.handle_completed_piece().await?;
        }
        self.pump_requests().await?;
        Ok(())
    }

    pub(crate) async fn handle_message(
        &mut self,
        message: Message,
    ) -> Result<session::Event, session::Error> {
        match message {
            Message::Bitfield(x) => {
                if self.bit_field.len() != x.len() {
                    panic!("Found a guy violating protocol, lmao")
                }
                self.bit_field.copy_from_slice(&x);
                Ok(Event::BitFieldUpdated)
            }
            Message::Choke => {
                self.is_choking = true;
                Ok(Event::ChokedMe)
            }
            Message::Unchoke => {
                self.is_choking = false;
                Ok(Event::UnchokedMe)
            }
            Message::Request {
                index,
                offset,
                length,
            } => {
                if !self.is_valid_block(index, offset, length).await {
                    return Ok(Event::Ignore);
                }
                Ok(Event::PieceRequested {
                    index,
                    offset,
                    length,
                })
            }
            Message::Have(x) => {
                self.update_bitfield(x)?;
                Ok(Event::Have(x))
            }
            Message::Interested => {
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
            Message::NotInterested => {
                self.is_interested = false;
                Ok(Event::PeerNotInterested)
            }
            Message::UnexpectedId(_) => return Err(Error::ProtocolViolation),
        }
    }
}
