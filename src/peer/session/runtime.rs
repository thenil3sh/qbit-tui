
use crate::peer::Message;
use crate::peer::PeerSession as Session;
use crate::peer::Piece;
use crate::peer::SessionError as Error;
use crate::peer::session;

impl Session {
    pub async fn run(&mut self) -> Result<(), Error> {
        self.send_bitfield().await?;
        self.connection.send(Message::Choke).await?;
        loop {
            tokio::select! {
                message = self.connection.read_message() => {
                    let message = message?;
                    let event = self.handle_message(message).await?;
                    self.handle_event(event).await?;
                }
                commit = self.commit_rx.recv() => {
                    self.handle_commit_event(commit.unwrap()).await?;
                }
            }
            self.try_reschedule().await?;
        }
    }

    /// Reschedules actions, whenever one of following event occurs :
    /// - Committer commits a piece (sent by any session)
    /// - A message is recieved (with-in current session)
    async fn try_reschedule(&mut self) -> session::Result<()> {
        if self.is_choking {
            return Ok(());
        }
        if self.current_piece.is_none() {
            // I'm interested, but I'm not supposed to
            if !self.should_be_interested().await {
                if self.am_interested {
                    self.am_interested = false;
                    self.connection.send(Message::NotInterested).await?;
                }
                return Ok(());
            }

            // I'm supposed to be interested, but I'm not
            if !self.am_interested {
                self.am_interested = true;
                self.connection.send(Message::Interested).await?;
            }

            // I'm interested, and I'm supposed to
            if let Some(index) = self.reserve_interesting_piece().await {
                self.current_piece = Some(Piece::new(index, self.torrent_info.piece_len(index)));
            }
        }
        self.pump_requests().await?;
        Ok(())
    }
}
