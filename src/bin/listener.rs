use std::io::{read_to_string, Read};

use bytes::{buf::Reader, BytesMut};
use qbit::tracker::{self, get_url};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

