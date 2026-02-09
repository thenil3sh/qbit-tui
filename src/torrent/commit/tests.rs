use std::{fs, sync::Arc};

use serial_test::serial;
use tempfile::TempDir;
use tokio::sync::{Mutex, mpsc};

use super::*;
use crate::torrent::{
    self, CommitJob, Committer, FileLayout, Metadata, State, info::NormalisedInfo,
};

async fn with_temp_committer<T, F, Fut>(metadata: &Metadata, f: F) -> T
where
    F: FnOnce(Committer, TempDir) -> Fut,
    Fut: Future<Output = T>,
{
    let temp_dir = TempDir::new().unwrap();
    let old_home = std::env::var("XDG_DATA_HOME");
    unsafe {
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());
    }
    let info = NormalisedInfo::try_from(metadata).unwrap().atomic();
    let file_layout = FileLayout::try_from(info.as_ref()).unwrap().atomic();
    let state = Arc::new(Mutex::new(State::try_from(metadata).unwrap()));

    let comitter = Committer::new(state, info.info_hash, info, file_layout);
    comitter.init_storage().await.unwrap();
    let result = f(comitter, temp_dir).await;

    unsafe {
        match old_home {
            Ok(x) => std::env::set_var("XDG_DATA_HOME", x),
            Err(_) => std::env::remove_var("XDG_DATA_HOME"),
        }
    }
    result
}

#[tokio::test]
#[serial]
async fn commit_single_piece_in_single_file() {
    let metadata = Metadata::fake();
    with_temp_committer(&metadata, |mut committer, temp_dir| async move {
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
                .join(committer.info_hash.to_owned().to_string())
                .join(&committer.info.name)
                .with_extension("tmp"),
        )
        .await
        .unwrap();
        assert_ne!(file, data.as_slice());
        assert_eq!(file[..data.len()], data);
    })
    .await;
}
