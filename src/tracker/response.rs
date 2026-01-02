use std::{any, io::Read};

use anyhow::{anyhow, bail};

use crate::tracker::peer::Peer;

#[derive(serde::Deserialize, Debug)]
pub struct Response {
    interval : u32,
    peers : Vec<Peer>
}

impl TryFrom<&[u8]> for Response {
    type Error = bendy::serde::Error;
    fn try_from(value: &[u8]) -> Result<Self, bendy::serde::Error> {
        bendy::serde::from_bytes(value.as_ref())
    }
}