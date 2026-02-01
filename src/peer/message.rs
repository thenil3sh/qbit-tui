use bytes::{BufMut, Bytes, BytesMut};
use std::{
    fmt::Debug,
    io::{self, Error, ErrorKind},
};

pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Bytes),
    Request {
        index: u32,
        offset: u32,
        length: u32,
    },
    Piece {
        index: u32,
        offset: u32,
        data: Bytes,
    },
    // Cancel,
    UnexpectedId(u8),
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::KeepAlive => f.write_str("KeepAlive"),
            Message::Choke => f.write_str("Choke"),
            Message::Unchoke => f.write_str("Unchoke"),
            Message::Bitfield(_) => f.write_str("Bitfield [...]"),
            Message::Interested => f.write_str("Interested"),
            Message::Have(x) => f.debug_tuple("Have").field(x).finish(),
            Message::NotInterested => f.write_str("Not Interested"),
            Message::Request {
                index,
                offset,
                length,
            } => f
                .debug_struct("Request")
                .field("index", index)
                .field("offset", offset)
                .field("length", length)
                .finish(),
            Message::Piece {
                index,
                offset,
                data,
            } => f
                .debug_struct("\x1b[32mPiece\x1b[0m")
                .field("index", index)
                .field("offset", offset)
                .field("data", &"[...]")
                .finish(),
            Message::UnexpectedId(i) => f.write_str(&format!("Unexpected Id : {i}")),
        }
    }
}

impl Message {
    pub fn decode(id: u8, payload: Bytes) -> io::Result<Self> {
        match id {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => Self::handle_have(payload),
            5 => Self::handle_bitfield(payload),
            6 => todo!("`Request`, isn't implemented yet"),
            7 => Self::handle_piece(payload),
            8 => todo!("`Cancel`, isn't implemented yet"),
            _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Id")),
        }
    }

    fn handle_have(payload: Bytes) -> io::Result<Self> {
        if payload.is_empty() || payload.len() != 4 {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Empty payload, expected Have Index",
            ))
        } else {
            let index = u32::from_be_bytes(
                payload[..4]
                    .try_into()
                    .expect("Failed to parse u32 index for Have Message"),
            );
            Ok(Self::Have(index))
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

    pub fn encode(&self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(self.encode_length());
        match self {
            Message::Choke => {
                bytes.put_u32(1);
                bytes.put_u8(0);
            }
            Message::Unchoke => {
                bytes.put_u32(1);
                bytes.put_u8(1);
            }
            Message::Interested => {
                bytes.put_u32(1);
                bytes.put_u8(2);
            }
            Message::NotInterested => {
                bytes.put_u32(1);
                bytes.put_u8(3);
            }
            Message::Have(x) => {
                bytes.put_u32(5);
                bytes.put_u8(4);
                bytes.put_u32(*x);
            }
            Message::Bitfield(bitfield) => {
                bytes.put_u32(bitfield.len() as u32 + 1);
                bytes.put_u8(5);
                bytes.put_slice(bitfield);
            }
            Message::Request {
                index,
                offset,
                length,
            } => {
                bytes.put_u32(13);
                bytes.put_u8(6);
                bytes.put_u32(*index);
                bytes.put_u32(*offset);
                bytes.put_u32(*length);
            }
            Message::Piece {
                index,
                offset,
                data,
            } => {
                bytes.put_u32(9 + data.len() as u32);
                bytes.put_u8(7);
                bytes.put_u32(*index);
                bytes.put_u32(*offset);
                bytes.put_slice(data);
            }
            Message::KeepAlive => {
                bytes.put_u32(0);
            }
            Message::UnexpectedId(_) => {
                panic!("No way you'll do that??")
            }
        }
        bytes.freeze()
    }

    fn encode_length(&self) -> usize {
        match self {
            Message::Choke | Message::Unchoke | Message::Interested | Message::NotInterested => 5,
            Message::Have(_) => 9,
            Message::Bitfield(bitfield) => 5 + bitfield.len(),
            Message::Request {
                index: _,
                offset: _,
                length: _,
            } => 17,
            Message::Piece {
                index: _,
                offset: _,
                data,
            } => 13 + data.len(),
            Message::KeepAlive => 1,
            Message::UnexpectedId(_) => 0,
        }
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

    #[test]
    fn encoding_messages_with_no_payload() {
        assert_eq!([0, 0, 0, 1, 0], Message::Choke.encode().as_ref());
        assert_eq!([0, 0, 0, 1, 1], Message::Unchoke.encode().as_ref());
        assert_eq!([0, 0, 0, 1, 2], Message::Interested.encode().as_ref());
        assert_eq!([0, 0, 0, 1, 3], Message::NotInterested.encode().as_ref());
    }

    #[test]
    fn encoding_request_messages() {
        let message = Message::Request {
            index: 3,
            offset: 4,
            length: 16384,
        };
        let expected_bytes = [0, 0, 0, 13, 6, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 64, 0];

        assert_eq!(expected_bytes, message.encode().as_ref())
    }

    #[test]
    fn encoding_message_with_bitfield() {
        let bitfield: Bytes = [0b10100011, 0b10100000].as_ref().into();
        let message = Message::Bitfield(bitfield.clone());
        let mut expected_bytes = vec![0, 0, 0, 3, 5];
        expected_bytes.extend_from_slice(bitfield.as_ref());
        assert_eq!(expected_bytes, message.encode().as_ref());
    }

    #[test]
    fn encoding_message_with_piece() {
        let piece: Bytes = [
            0b10100011, 0b10100000, 0b10100011, 0b10100000, 0b10100011, 0b10100000,
        ]
        .as_ref()
        .into();
        let message = Message::Piece {
            index: 3,
            offset: 4,
            data: piece.clone(),
        };

        let mut expected_bytes = vec![0, 0, 0, 15, 7, 0, 0, 0, 3, 0, 0, 0, 4];
        expected_bytes.extend_from_slice(piece.as_ref());

        assert_eq!(expected_bytes, message.encode().as_ref());
    }
}
