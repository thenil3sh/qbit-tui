use crate::peer::{id, message, Connection};
use bytes::Bytes;
use serde::de::Unexpected;
use std::io::{self, Error, ErrorKind};

#[derive(Debug)]
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    // Have,
    Bitfield(Bytes),
    // Request,
    Piece {
        index: u32,
        offset: u32,
        data: Bytes,
    },
    // Cancel,
    UnexpectedId(u8),
}

impl Message {
    pub(crate) fn decode(id: u8, payload: Bytes) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => todo!("`Have` isn't implemented yet"),
            5 => Self::handle_bitfield(payload),
            6 => todo!("`Request`, isn't implemented yet"),
            7 => Self::handle_piece(payload),
            8 => todo!("`Cancel`, isn't implemented yet"),
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Id")),
        }
    }

    fn handle_bitfield(payload: Bytes) -> io::Result<Self> {
        if payload.is_empty() {
            Err(Error::new(ErrorKind::InvalidData, "Empty Bitfield"))
        } else {
            Ok(Self::Bitfield(payload))
        }
    }

    fn handle_piece(payload: Bytes) -> io::Result<Self> {
        if payload.len() <= 8 {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid Piece"));
        }
        let index = u32::from_be_bytes(payload[0..4].try_into().unwrap());
        let offset = u32::from_be_bytes(payload[4..8].try_into().unwrap());
        let data = payload.slice(8..);

        Ok(Self::Piece {
            index,
            offset,
            data,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::peer::Message;
    use bytes::Bytes;

    #[test]
    fn parsing_valid_message() {
        Message::decode(3, Bytes::new()).unwrap();
    }

    #[test]
    fn parsing_a_message() {
        let message = Message::Bitfield(b"".as_ref().into());
    }

    #[test]
    fn decoding_invalid_message_id() {
        assert!(Message::decode(9, Bytes::new()).is_err());
    }

    #[test]
    fn decoding_empty_piece() {
        let bytes = Bytes::new();
        assert!(Message::decode(7, bytes).is_err());
    }

    #[test]
    fn decoding_empty_bitfield() {
        assert!(Message::decode(5, Bytes::new()).is_err());
    }
    
    #[test]
    fn decoding_valid_bitfield() {
        Message::decode(5, vec![1u8; 8].into()).unwrap();
    }
    
    #[test]
    fn decoding_valid_piece() {
        Message::decode(7, vec![0u8; 10].into()).unwrap();
    }
}
