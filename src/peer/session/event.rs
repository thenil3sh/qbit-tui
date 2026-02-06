use bytes::Bytes;

#[derive(Debug)]
pub enum Event {
    BitFieldUpdated,
    ChokedMe,
    UnchokedMe,
    PeerInterested,
    PieceRecieved { index : u32, offset : u32, data : Bytes },
    Have(u32),
    KeepAlive,
    PeerNotInterested,
    PieceRequested { index : u32, offset : u32, length : u32 },
    Ignore,
}