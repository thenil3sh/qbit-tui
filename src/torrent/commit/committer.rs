use super::{error::Result, job::Job};
use bytes::Bytes;
use std::{
    env, io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs,
    io::AsyncSeekExt,
    sync::{Mutex, mpsc},
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::torrent::{self, Info, InfoHash, commit};

pub struct Committer {
    sender: mpsc::Sender<Job>,
    reciever: mpsc::Receiver<Job>,
    state: Arc<Mutex<torrent::State>>,
    info_hash: InfoHash,
    info: Arc<Info>,
}

impl<'d> Committer {
    pub fn new(state: Arc<Mutex<torrent::State>>, info_hash: InfoHash, info: Arc<Info>) -> Self {
        let (sender, reciever) = mpsc::channel(todo!());
        Self {
            sender,
            reciever,
            state,
            info_hash,
            info,
        }
    }

    /// Gives out a cloned copy of sender
    pub fn sender(&self) -> mpsc::Sender<Job> {
        self.sender.clone()
    }

    pub async fn commit_path(&self) -> io::Result<PathBuf> {
        let path = if let Ok(x) = env::var("HOME") {
            PathBuf::new().join(x)
        } else {
            panic!("variable HOME not set");
        }
        .join(".qbit")
        .join(self.info_hash.to_string());
        fs::create_dir_all(path.clone()).await?;
        Ok(path.to_owned().join(self.info.name.as_str()))
    }

    async fn pre_allocate(&self, path: &mut PathBuf) -> io::Result<()> {
        path.set_extension("tmp");
        if !path.exists() {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)
                .await?;
            file.set_len(self.info.length as u64).await?;
        }
        Ok(())
    }

    pub async fn commit(&mut self, job: Job) -> Result<()> {
        let mut path = self.commit_path().await?.join(self.info.name.as_str());
        if !path.exists() {
            self.pre_allocate(&mut path).await?;
        }

        let mut file = OpenOptions::new().write(true).open(path).await?;

        let absolute_index = (job.index * self.info.piece_len(job.index)) as u64;

        file.seek(io::SeekFrom::Start(absolute_index)).await?;
        file.write_all(&job.bytes).await?;
        
        Ok(())
    }

    pub async fn run(&mut self) {
        while let Some(job) = self.reciever.recv().await {
            if let Err(err) = self.commit(job).await {
                eprintln!("{err}");
            }
        }
    }
}
