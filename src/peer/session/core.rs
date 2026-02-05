use std::sync::Arc;

use tokio::sync::{Mutex, broadcast, mpsc};

use crate::{
    peer::{session, Connection, Message, Piece},
    torrent::{self, commit, CommitEvent, CommitJob},
};

pub struct Session {
    pub(crate) commit_tx: mpsc::Sender<CommitJob>,
    pub(crate) commit_rx: broadcast::Receiver<commit::Event>,
    pub(crate) connection: Connection,
    pub(crate) torrent_info: Arc<torrent::Info>,
    pub(crate) current_piece: Option<Piece>,
    pub(crate) state: Arc<Mutex<torrent::State>>,
    pub(crate) is_choking: bool,
    pub(crate) is_interested: bool,
    pub(crate) am_choking: bool,
    pub(crate) am_interested: bool,
    pub(crate) bit_field: Option<Vec<u8>>,
}

impl Session {
    pub fn new(
        connection: Connection,
        torrent_info: Arc<torrent::Info>,
        state: Arc<Mutex<torrent::State>>,
        commit_tx: mpsc::Sender<CommitJob>,
        commit_rx: broadcast::Receiver<commit::Event>,
    ) -> Self {
        Self {
            commit_tx,
            commit_rx,
            connection,
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
}


impl Session {
    pub(crate) async fn handle_commit_event(
        &mut self,
        event: commit::Event,
    ) -> session::Result<()> {
        match event {
            CommitEvent::PieceCommit(index) => self.connection.send(Message::Have(index)).await?,
            CommitEvent::FailedCommit => todo!("Fokin failed?"),
        }
        Ok(())
    }

    /// Repeatedly places piece block requests in pipeline, upto Piece's on-fly capacity
    /// So let's say if Piece has capacity of handling 4 blocks on-fly, only 4 blocks will be asked at a time
    pub(crate) async fn pump_requests(&mut self) -> session::Result<()> {
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
            // eprintln!("{piece_request:?} SENT");
            self.connection.send(piece_request).await?;
        }
        Ok(())
    }
}
