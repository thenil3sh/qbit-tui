pub fn get_url(torrent : Torrent) -> String {
    format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}",
        torrent.announce,
        torrent.info_hash.to_url_encoded(),
        PeerId::new().url_encoded(),
        6881,
        0,
        0,
        torrent.info.length
    )
}

pub mod peer;
pub use peer::PeerId;

use crate::torrent::Metadata as Torrent;
