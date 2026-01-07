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

#[allow(unused)]
mod test {
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, task::JoinHandle};
    use std::net::{IpAddr, SocketAddr};

    use crate::tracker;
    
    const VALID_RESPONSE_BODY : &[u8] = b"d8:intervali900e5:peersld2:ip11:77.79.169.44:porti18820eed2:ip12:82.197.75.394:porti40604eed2:ip14:64.135.236.1424:porti55304eed2:ip12:176.102.77.94:porti50413eed2:ip12:51.154.2.2404:porti12102eed2:ip14:193.77.135.1814:porti65382eed2:ip14:146.70.194.1084:porti56116eed2:ip13:70.175.76.2394:porti22222eed2:ip13:62.169.136.764:porti6881eeee";
    
    async fn server_that_responds_with_valid_response_body() -> (JoinHandle<()>, SocketAddr){
        let listener = TcpListener::bind("127.0.0.0:0").await.unwrap();
        let server_addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut resp_buffer = [0u8; 1024];
            stream.read(&mut resp_buffer).await.unwrap();
            
            stream.write_all(VALID_RESPONSE_BODY).await.unwrap();
        });
        
        (handle, server_addr)
    }
    
    #[tokio::test]
    async fn send_get_request_and_parse_response() {
        let (handle, server_addr) = server_that_responds_with_valid_response_body().await;
        
        let mut peer_stream = TcpStream::connect(server_addr).await.unwrap();
        peer_stream.write_all(b"lmao bro get owned").await.unwrap();
        handle.await.unwrap();
        
        let mut response_buffer = [0u8; 1024];
        peer_stream.read(&mut response_buffer).await.unwrap();
        
        assert!(bendy::serde::from_bytes::<tracker::Response>(&response_buffer).is_ok());
    }
}
