use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
    time::SystemTime,
};

use bytes::{Bytes, BytesMut};

use crate::{
    cache::{self, CacheType},
    torrent::InfoHash,
};

pub struct Cache {
    r#type: cache::CacheType,
    info_hash: InfoHash,
    path: PathBuf,
}

impl Cache {
    pub fn new(r#type: CacheType, info_hash: InfoHash) -> io::Result<Self> {
        let cache_dir = Cache::directory().join("qbit-tui");
        let path = match r#type {
            CacheType::TrackerResponse => cache_dir
                .join("tracker-response")
                .join(info_hash.to_hex_lower()),
        };
        let cache = Self {
            info_hash: info_hash,
            r#type,
            path,
        };
        cache.load_or_create()?;
        Ok(cache)
    }

    pub fn len(&self) -> u64 {
        self.path.metadata().expect("Failed to get cache metadata").len()
    }

    pub fn read(&self) -> Bytes {
        let mut file = fs::OpenOptions::new().read(true).open(&self.path).expect("Failed to read file");
        let mut bytes = BytesMut::with_capacity(self.len() as usize);
        bytes.resize(self.len() as usize, 0u8);
        file.read_exact(&mut bytes);

        bytes.freeze()
    }

    fn directory() -> PathBuf {
        if let Ok(dir) = env::var("XDG_CACHE_HOME") {
            PathBuf::from(dir)
        } else if let Ok(dir) = env::var("HOME") {
            PathBuf::from(dir).join(".cache")
        } else {
            panic!("Variables HOME and XDG_CACHE_HOME are not set");
        }
    }

    fn load_or_create(&self) -> io::Result<File> {
        match self.r#type {
            CacheType::TrackerResponse => self.tracker_response(),
        }
    }

    fn tracker_response(&self) -> io::Result<File> {
        if let Some(path) = self.path.parent() {
            fs::create_dir_all(path)?;
        }

        fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.path)
    }

    pub fn is_fresher_than<T>(&self, seconds: T) -> bool
    where
        T: Into<u64>,
    {
        let cache_meta = self.path.metadata().expect("Failed to fetch file MetaData");
        let last_modified = cache_meta
            .modified()
            .expect("Failed to get cache last modified");
        let time_since_modified = SystemTime::now()
            .duration_since(last_modified)
            .expect("Duration subtraction error")
            .as_secs();

        return time_since_modified <= seconds.into();
    }

    pub fn is_empty(&self) -> bool {
        let file_length = self
            .path
            .metadata()
            .expect("Failed to fetch file Metadata")
            .len();
        file_length == 0
    }

    pub fn update<T>(&self, data: T) -> io::Result<()>
    where
        T: AsRef<[u8]>,
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        file.write_all(data.as_ref())?;
        Ok(())
    }
}
