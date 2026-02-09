use crate::peer::{session::{self, Error}, PeerSession as Session};

impl Session {
    /// Looks for a piece in peer's bitfield, if there's anything interesting, it'll reserve it, then return the index to user
    /// Else it responds with None
    /// > NOTE: Global piece reservation is temporary.
    /// > This will be removed once upload + choking are stable
    pub(crate) async fn reserve_interesting_piece(&self) -> Option<u32> {
        let mut state = self.state.lock().await;
        for (byte_idx, (peer, mine)) in self
            .bit_field
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

    /// Update peer's bitfield, helps keeping track of peer's bitfield
    /// 
    /// ## Error
    /// Fails when Piece requested larger in size than bitfields is supposed to hold
    pub(crate) fn update_bitfield(&mut self, index: u32) -> session::Result<()> {
        let piece = index as usize;
        let byte = piece / 8;
        if byte >= self.bit_field.len() {
            return Err(Error::BadRequest);
        }
        let bit = piece % 8;

        let mask = 1 << (7 - bit);

        self.bit_field[byte] |= mask;
        Ok(())
    }

    /// Only checks if I have to be interested in peer.
    /// To reserve an interesting piece, use `reserve_interesting_piece()` instead
    pub(crate) async fn should_be_interested(&self) -> bool {
        let my_state = self.state.lock().await;
        my_state
            .bit_field
            .iter()
            .zip(self.bit_field.iter())
            .any(|(mine, peer)| !mine & peer != 0)
    }

    pub(crate) async fn is_valid_block(&self, index : u32, offset : u32, length : u32) -> bool {
        if self.am_choking {
            return false;
        }
        {
            let piece_len = self.torrent_info.piece_len(index);
            let state = self.state.lock().await;
            if !state.have_piece(index) || piece_len < offset + length {
                return false;
            }
        }
        return true;
    }
}
