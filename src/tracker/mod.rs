use crate::peer;
pub mod response;
use bytes::Bytes;
use crate::torrent::Metadata as Torrent;
pub use response::Response;



pub fn get_url(torrent : Torrent) -> String {
    format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}",
        torrent.announce,
        torrent.info_hash.to_url_encoded(),
        peer::ID.url_encoded(),
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

mod test {
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, task::JoinHandle};
    use std::net::SocketAddr;

    use crate::tracker;
    
    const VALID_RESPONSE_BODY : &[u8] = b"d8:intervali900e5:peersld2:ip11:77.79.169.44:porti18820eed2:ip12:82.197.75.394:porti40604eed2:ip14:64.135.236.1424:porti55304eed2:ip12:176.102.77.94:porti50413eed2:ip12:51.154.2.2404:porti12102eed2:ip14:193.77.135.1814:porti65382eed2:ip14:146.70.194.1084:porti56116eed2:ip13:70.175.76.2394:porti22222eed2:ip13:62.169.136.764:porti6881eeee";

    
    async fn start_server() -> (JoinHandle<()>, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();
        let handle = tokio::spawn(async move {
            let mut request_buffer = [0u8; 1024];
            stream.read(&mut request_buffer).await.unwrap();
            stream.write_all(VALID_RESPONSE_BODY).await.unwrap();
        });
        return (handle, listener.accept().await.unwrap().1);
    }
    
    #[tokio::test]
    async fn loook() {
        
        let (handle, addr) = start_server().await;
        
        let mut peer_stream = TcpStream::connect(addr).await.unwrap();
        peer_stream.write_all(b"just some random text").await.unwrap();
        
        let mut tracker_response = vec![0u8; 1024];
        peer_stream.read(&mut tracker_response).await.unwrap();
        
        let response : Result<tracker::Response, bendy::serde::Error> = bendy::serde::from_bytes(&tracker_response);
        assert!(response.is_err());
    }
}
