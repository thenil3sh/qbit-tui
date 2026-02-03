#[derive(Clone, Debug)]
pub enum Event {
    PieceCommit(u32),
    FailedCommit,
}