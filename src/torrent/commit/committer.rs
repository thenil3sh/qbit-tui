use super::job::Job;
use std::{io::SeekFrom, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    fs,
    io::AsyncSeekExt,
    sync::{Mutex, broadcast, mpsc},
};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{peer::Message, torrent::{
    self, commit::{self, Error, Event}, CommitEvent, Info, InfoHash
}};

pub struct Committer {
    sender: mpsc::Sender<Job>,
    reciever: mpsc::Receiver<Job>,
    state: Arc<Mutex<torrent::State>>,
    info_hash: InfoHash,
    info: Arc<Info>,
    broadcast : broadcast::Sender<commit::Event>,
}

impl Committer {
    pub fn new(state: Arc<Mutex<torrent::State>>, info_hash: InfoHash, info: Arc<Info>) -> Self {
        let (sender, reciever) = mpsc::channel(4);
        Self {
            sender,
            reciever,
            state,
            info_hash,
            info,
            broadcast : broadcast::Sender::new(3)
        }
    }

    /// Gives out a cloned copy of sender
    pub fn sender(&self) -> mpsc::Sender<Job> {
        self.sender.clone()
    }

    /// Allocates storage to a file, if it doesn't already
    async fn init_storage(&self) -> commit::Result<()> {
        let path = self.base_dir()?;
        fs::create_dir_all(&path).await?;

        let path = path.join(self.info.name.as_str()).with_extension("tmp");
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .await?;
        file.set_len(self.info.length as u64).await?;
        Ok(())
    }

    pub fn base_dir(&self) -> commit::Result<PathBuf> {
        let path = dirs::data_dir()
            .ok_or(Error::BaseDirectoryNotFound)?
            .join(".qbit")
            .join(self.info_hash.to_string());
        Ok(path)
    }

    fn path(&self) -> commit::Result<PathBuf> {
        let mut path = self
            .base_dir()
            .expect("storage must be initialized first")
            .join(self.info.name.as_str());
        path.set_extension("tmp");
        Ok(path)
    }

    pub async fn commit(&mut self, job: &Job) -> commit::Result<()> {
        let path = self.path()?;
        let mut file = OpenOptions::new().write(true).open(path).await?;

        let absolute_index = (self.info.piece_length * job.index) as u64;
        file.seek(SeekFrom::Start(absolute_index)).await?;

        file.write_all(&job.bytes).await?;
        self.state.lock().await.mark_piece_complete(job.index);
        self.state.lock().await.remove_in_flight(job.index);
        Ok(())
    }

    pub async fn run(&mut self) -> commit::Result<()> {
        self.init_storage().await?;

        while let Some(job) = self.reciever.recv().await {
            let mut attempts = 4;
            while let Err(err) = self.commit(&job).await
                && attempts > 0
            {
                eprintln!("Err : {err} | Failed to commit {}", job.index);
                tokio::time::sleep(Duration::from_millis(200)).await;
                attempts -= 1;
            }
            if attempts > 0 {
                self.broadcast.send(Event::PieceCommit(job.index))?;
            } else {
                self.broadcast.send(Event::FailedCommit)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tokio::sync::mpsc;

    use crate::torrent::{self, CommitJob, Committer, Metadata, State};


    /// Makes a random temp directory and simulates XDG_DATA_HOME within the environment.
    /// Because it is making use of global env var, should be only taken in use in single single thread,
    async fn with_temp_dir<F, T, Fut>(f: F) -> T
    where
        F: FnOnce(mpsc::Sender<CommitJob>) -> Fut,
        Fut: Future<Output = T>,
    {
        let temp = tempfile::TempDir::new().unwrap();
        let old_entry = std::env::var("XDG_DATA_HOME");
        let metadata = Metadata::fake();

        
        
        let result;
        unsafe {
            std::env::set_var("XDG_DATA_HOME", temp.path());
            let state = State::load_or_new(&metadata).await.atomic();
            let committer = Committer::new(state, metadata.info_hash, metadata.info.atomic());
            result = f(committer.sender.clone()).await;

            tokio::spawn(async move {
                committer.init_storage().await.unwrap();
                // while 
            });

            
            match old_entry {
                Ok(old) => std::env::set_var("XDG_DATA_HOME", old),
                Err(_) => std::env::remove_var("XDG_DATA_HOME"),
            }
        };
        result
    }
    

    #[tokio::test]
    #[serial_test::serial]
    async fn committer_successfully_writes_to_disk() {
        with_temp_dir(|sender| async {
            
        }).await;
    }
}
