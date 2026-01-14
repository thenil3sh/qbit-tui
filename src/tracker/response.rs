use std::{fs, io};

use crate::{torrent, tracker::peer::Peer};

#[derive(serde::Deserialize, Debug)] //////////////////////////////////////// only now
#[cfg_attr(test, derive(PartialEq))]
pub struct Response {
    pub interval: u32,
    pub peers: Vec<Peer>,
}

impl Response {
    fn new(torrent : &torrent::Metadata) {

    }
}

impl TryFrom<&[u8]> for Response {
    type Error = bendy::serde::Error;
    fn try_from(value: &[u8]) -> Result<Self, bendy::serde::Error> {
        bendy::serde::from_bytes(value)
    }
}

#[allow(unused)]
mod test {

    use super::{Response, Peer};
    use std::net::Ipv4Addr;

    #[test]
    fn parsing_valid_response() {
        let response =
            b"d8:intervali900e5:peersld2:ip7:0.0.0.04:porti3421eed2:ip7:1.2.3.44:porti1234eeee";
        let expected_response = Response {
            interval: 900,
            peers: vec![
                Peer { ip : Ipv4Addr::new(0,0,0,0), port : 3421, id : None },
                Peer { ip : Ipv4Addr::new(1,2,3,4), port : 1234, id : None },
            ],
        };

        let parsed_response : Response = bendy::serde::from_bytes(response).unwrap();

        assert_eq!(expected_response, parsed_response);
    }

    #[test]
    fn parsing_invalid_response() {
        let response =
            b"d8:intervali900e5:peersld2:ip7:0.0.0.04:porti3421eed2:ip7:1.2.3.44:porti1234ee";
        
        assert!(bendy::serde::from_bytes::<Response>(response).is_err());
    }


}
