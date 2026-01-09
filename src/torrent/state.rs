use std::collections::HashSet;

use bytes::Bytes;

pub struct State {
    pub(crate) bit_field : Vec<u8>,
    in_flight : HashSet<u32>,
    num_pieces : usize
}