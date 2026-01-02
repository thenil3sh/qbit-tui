
pub mod peer;
pub mod response;
use bytes::Bytes;
use crate::torrent::Metadata as Torrent;
pub use response::Response;



pub fn get_url(torrent : Torrent) -> String {
    format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}",
        torrent.announce,
        torrent.info_hash.to_url_encoded(),
        peer::Id::new().url_encoded(),
        6881,
        0,
        0,
        torrent.info.length
    )
}

pub async fn fetch_tracker_bytes<T>(url: T) -> anyhow::Result<Bytes>
where
    T: reqwest::IntoUrl,
{
    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?;
    Ok(bytes)
}

