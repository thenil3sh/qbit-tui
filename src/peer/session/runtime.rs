use std::time::Duration;

use crate::peer::session;
use crate::peer::PeerSession as Session;
use crate::peer::Piece;
use crate::peer::SessionError as Error;

impl Session {
    pub async fn run(&mut self) -> Result<(), Error> {
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
                _ = tokio::time::sleep(Duration::from_secs(120)) => {
                    return Err(Error::TimeOut)
                }
            }
        }
    }
    
    #[allow(unused)]
    async fn try_reschedule(&mut self) -> session::Result<()> {
        if self.current_piece.is_none() && !self.is_choking && self.am_interested {
            if let Some(index) = self.reserve_interesting_piece().await {
                self.current_piece = Some(Piece::new(index, self.torrent_info.piece_len(index)));
                self.pump_requests().await?;
            }
        }
        Ok(())
    }
}
