use std::{fs::File, io::{BufWriter, Read, Write}};

use ciborium::into_writer;
use qbit::torrent::State;
use tokio::{ io::{AsyncWriteExt}};

#[tokio::main]
async fn main() {
    {
        let state = State::new();
        let file = File::create("oreo.txt").unwrap();
        let mut writer = BufWriter::new(file);
        
        into_writer(&state, &mut writer).unwrap();
        writer.flush().unwrap(); 
    }

    {
        let file = File::open("")



    }

    
}