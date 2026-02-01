use bytes::Bytes;

pub struct Job {
    pub(crate) index : u32,
    pub(crate) bytes : Bytes
}

impl Job {
    fn new(index : u32, bytes : Bytes) -> Self {
        Self { index, bytes }
    }
}