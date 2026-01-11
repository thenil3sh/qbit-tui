use qbit::{peer::{Handshake, Message}, torrent, tracker};
use std::{env, net::SocketAddr, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, task::JoinSet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arg = env::args().nth(1).unwrap_or_else(|| "test/debian.torrent".to_string());

    let torrent = torrent::Metadata::from_file(&arg)?;
    let info_hash = torrent.info_hash.clone();

    eprintln!("Using torrent: {} (announce: {})", arg, torrent.announce);

    let url = tracker::get_url(torrent);
    eprintln!("Announcing to tracker: {}", url);

    let bytes = tracker::fetch_tracker_bytes(url).await?;
    let response: tracker::Response = (&bytes[..]).try_into()?;

    eprintln!("Found {} peers, testing (first 50)...", response.peers.len());

    let mut set = JoinSet::new();

    for (i, peer) in response.peers.into_iter().enumerate().take(50) {
        let info_hash = info_hash.clone();
        set.spawn(async move {
            let res = tokio::time::timeout(Duration::from_secs(8), async move {
                let addr = SocketAddr::new(peer.ip.into(), peer.port);
                match TcpStream::connect(addr).await {
                    Ok(mut stream) => {
                        // send handshake
                        let hs = Handshake::from(&info_hash);
                        stream.write_all(hs.bytes()).await?;

                        // read handshake response (68 bytes)
                        let mut resp = [0u8; 68];
                        stream.read_exact(&mut resp).await?;

                        // wait for peer bitfield/have to decide interest
                        let mut desired_piece: Option<u32> = None;

                        // read length (4 bytes)
                        let mut lenb = [0u8; 4];
                        stream.read_exact(&mut lenb).await?;
                        let len = u32::from_be_bytes(lenb);

                        if len == 0 {
                            eprintln!("peer {i:2} @ {addr} -> KeepAlive");
                            return Ok::<(), anyhow::Error>(());
                        }

                        // read id
                        let mut idb = [0u8; 1];
                        stream.read_exact(&mut idb).await?;
                        let id = idb[0];

                        let payload_len = (len - 1) as usize;
                        let mut payload = vec![0u8; payload_len];
                        if payload_len > 0 {
                            stream.read_exact(&mut payload).await?;
                        }

                        match Message::decode(id, payload.into()) {
                            Ok(Message::Bitfield(bits)) => {
                                // find first set bit (MSB first per spec)
                                let mut found: Option<u32> = None;
                                for (bi, &b) in bits.as_ref().iter().enumerate() {
                                    for bit in 0..8 {
                                        let mask = 0x80u8 >> bit;
                                        if b & mask != 0 {
                                            found = Some((bi * 8 + bit) as u32);
                                            break;
                                        }
                                    }
                                    if found.is_some() { break; }
                                }

                                if let Some(idx) = found {
                                    eprintln!("peer {i:2} @ {addr} -> has piece {idx}, sending Interested");
                                    desired_piece = Some(idx);
                                    stream.write_all(Message::Interested.encode().as_ref()).await?;
                                } else {
                                    eprintln!("peer {i:2} @ {addr} -> bitfield had no pieces we want");
                                    stream.write_all(Message::NotInterested.encode().as_ref()).await?;
                                    return Ok::<(), anyhow::Error>(());
                                }
                            }
                            Ok(Message::Unchoke) => {
                                // peer unchoked immediately
                                eprintln!("peer {i:2} @ {addr} -> Unchoked (no prior bitfield)");
                            }
                            Ok(_) => {
                                eprintln!("peer {i:2} @ {addr} -> Other initial message");
                            }
                            Err(e) => {
                                eprintln!("peer {i:2} @ {addr} -> Failed to decode message: {e}");
                                return Ok::<(), anyhow::Error>(());
                            }
                        }

                        // wait for Unchoke to then request piece
                        if desired_piece.is_some() {
                            loop {
                                // read next message
                                let mut lenb = [0u8; 4];
                                stream.read_exact(&mut lenb).await?;
                                let len = u32::from_be_bytes(lenb);
                                if len == 0 {
                                    continue; // keepalive
                                }
                                let mut idb = [0u8; 1];
                                stream.read_exact(&mut idb).await?;
                                let id = idb[0];
                                let payload_len = (len - 1) as usize;
                                let mut payload = vec![0u8; payload_len];
                                if payload_len > 0 {
                                    stream.read_exact(&mut payload).await?;
                                }
                                match Message::decode(id, payload.into()) {
                                    Ok(Message::Unchoke) => {
                                        if let Some(idx) = desired_piece {
                                            eprintln!("peer {i:2} @ {addr} -> Unchoked, requesting piece {idx}");
                                            let req = Message::Request { index: idx, offset: 0, length: 16384u32 };
                                            stream.write_all(req.encode().as_ref()).await?;
                                        }
                                        break;
                                    }
                                    Ok(msg) => eprintln!("peer {i:2} @ {addr} -> msg: {:?}", msg),
                                    Err(e) => {
                                        eprintln!("peer {i:2} @ {addr} -> Failed to decode message: {e}");
                                        break;
                                    }
                                }
                            }
                        }

                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("peer {i:2} failed to connect: {e}");
                        Err(anyhow::anyhow!(e))
                    }
                }
            })
            .await;

            if res.is_err() {
                eprintln!("peer {i:2} timed out");
            }
        });
    }

    while let Some(_) = set.join_next().await {}

    Ok(())
}

///// 