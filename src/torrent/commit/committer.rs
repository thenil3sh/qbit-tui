use super::job::Job;
use bendy::encoding::SingleItemEncoder;
use rand::rand_core::le;
use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{fs::{OpenOptions, read}, io::AsyncWriteExt};
use tokio::{
    fs::{self, File},
    io::{self, AsyncSeekExt},
    sync::{Mutex, broadcast, mpsc},
};

use crate::{
    peer::Message,
    torrent::{
        self, CommitEvent, FileLayout, Info, InfoHash,
        commit::{self, Error, Event},
        info::{FileMode, NormalisedInfo},
    },
};

pub struct Committer {
    sender: mpsc::Sender<Job>,
    reciever: mpsc::Receiver<Job>,
    state: Arc<Mutex<torrent::State>>,
    pub(crate) info_hash: InfoHash,
    info: Arc<NormalisedInfo>,
    broadcast: broadcast::Sender<commit::Event>,
    file_layout: Arc<FileLayout>,
}

pub(crate) async fn init_file<T>(file_path: T, length: u64) -> io::Result<()>
where
    T: AsRef<Path>,
{
    let path = file_path.as_ref().parent().expect("");
    fs::create_dir_all(path).await?;

    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .await?;
    file.set_len(length).await
}

impl Committer {
    pub fn new(
        state: Arc<Mutex<torrent::State>>,
        info_hash: InfoHash,
        info: Arc<NormalisedInfo>,
        file_layout: Arc<FileLayout>,
    ) -> Self {
        let (sender, reciever) = mpsc::channel(8);
        let (broadcast, _) = broadcast::channel(16);
        Self {
            sender,
            reciever,
            state,
            info_hash,
            info,
            broadcast,
            file_layout,
        }
    }

    /// Clones a reciever for Commit::Event
    /// this listener can be awaited to be notified for any commit event,
    /// that happens during comitter's lifetime
    pub fn listener(&self) -> broadcast::Receiver<commit::Event> {
        self.broadcast.subscribe()
    }

    /// Gives out a cloned copy of sender
    pub fn sender(&self) -> mpsc::Sender<Job> {
        self.sender.clone()
    }

    /// Allocates storage to a file, if it doesn't already exist
    async fn init_storage(&self) -> commit::Result<()> {
        let path = self.base_dir()?;
        fs::create_dir_all(&path).await?;

        match self.info.file_mode.as_ref() {
            FileMode::Single { length } => {
                let path = path.join(self.info.name.as_str()).with_extension("tmp");
                init_file(path, *length).await?;
            }
            FileMode::Multiple { files } => {
                for f in files {
                    let path = path
                        .join(f.path.iter().fold(path.clone(), |x, y| x.join(y)))
                        .with_extension("tmp");
                    init_file(path, f.length).await?;
                }
            }
        }
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
        let piece_start = job.index as u64 * self.info.piece_length as u64;
        let piece_end = piece_start + job.bytes.len() as u64;

        for file in self.file_layout.files.iter() {
            let file_start = file.offset;
            let file_end = file_start + file.length;

            let write_start = piece_start.max(file_start);
            let write_end = piece_end.min(file_end);

            if write_start >= write_end {
                continue;
            }

            let file_offset = write_start - file_start;
            let piece_offset = write_start - piece_start;
            let write_length = write_end - write_start;

            let mut f = fs::OpenOptions::new().write(true).open(&file.path).await?;

            let start = piece_offset as usize;
            let end = start + write_length as usize;

            f.seek(SeekFrom::Start(file_offset)).await?;
            f.write_all(&job.bytes[start..end]).await?;
        }

        Ok(())
    }

    pub async fn update_save_state(&self, index: u32) -> commit::Result<()> {
        let mut state = self.state.lock().await;
        state.mark_piece_complete(index);
        state.save().await?;
        Ok(())
    }

    /// # Committer runtime
    /// - Initiates storage
    /// - Handles commit requests from sessions
    /// - If commits are success, updates the state
    /// - Then, notifies all the active sessions
    pub async fn run(&mut self) -> commit::Result<()> {
        self.init_storage().await?;

        while let Some(job) = self.reciever.recv().await {
            eprintln!("\x1b[33mCOMMITTING PIECE : {}\x1b[0m", job.index);
            let mut attempts = 4;
            while let Err(err) = self.commit(&job).await
                && attempts > 0
            {
                eprintln!("Err : {err} | Failed to commit {}", job.index);
                tokio::time::sleep(Duration::from_millis(200)).await;
                attempts -= 1;
            }
            if attempts > 0 {
                self.update_save_state(job.index).await?;
                self.broadcast.send(Event::PieceCommit(job.index))?;
            } else {
                self.broadcast.send(Event::FailedCommit)?;
            }
        }
        eprintln!("\x1b[31mCommitter EXITing, all senders dropped");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{fs, sync::Arc};

    use serial_test::serial;
    use tempfile::TempDir;
    use tokio::sync::{Mutex, mpsc};

    use crate::torrent::{
        self, CommitJob, Committer, FileLayout, Metadata, State, info::NormalisedInfo,
    };

    fn make_comitter(temp_dir: &TempDir, metadata: &Metadata) -> Committer {
        let old_home = std::env::var("XDG_DATA_HOME");
        unsafe {
            std::env::set_var("XDG_DATA_HOME", temp_dir.path());
        }
        let info = NormalisedInfo::try_from(&metadata.info).unwrap().atomic();
        let file_layout = FileLayout::try_from(info.as_ref()).unwrap().atomic();
        let state = Arc::new(Mutex::new(State::try_from(metadata).unwrap()));

        let comitter = Committer::new(state, info.info_hash, info, file_layout);
        unsafe {
            match old_home {
                Ok(x) => std::env::set_var("XDG_DATA_HOME", x),
                Err(_) => std::env::remove_var("XDG_DATA_HOME"),
            }
        }
        comitter
    }

    #[tokio::test]
    #[serial]
    async fn commit_single_piece_in_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let metadata = Metadata::fake();

        let mut committer = make_comitter(&temp_dir, &metadata);
        committer.init_storage().await.unwrap();

        let data = vec![0xAB; 1024];
        let job = CommitJob {
            index: 0,
            bytes: data.clone().into(),
        };

        committer.commit(&job).await.unwrap();

        let file = tokio::fs::read(
            temp_dir
                .path()
                .join(".qbit")
                .join(committer.info_hash.to_string())
                .join(&committer.info.name)
                .with_extension("tmp"),
        ).await.unwrap();
        
        assert_eq!(file, data);
    }
}
