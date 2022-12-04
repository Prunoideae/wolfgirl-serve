use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use chrono::Utc;
use dashmap::DashMap;
use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DownloadCount {
    downloads: DashMap<PathBuf, AtomicUsize>,
}

impl DownloadCount {
    pub fn new() -> Self {
        DownloadCount {
            downloads: DashMap::new(),
        }
    }

    /// Get the download count of a path in usize
    pub fn get_count(&self, path: &PathBuf) -> usize {
        self.downloads
            .get(path)
            .map(|num| num.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Increases the download counter of corresponding path by 1.
    /// Returns the download count after added.
    pub fn incr_count(&self, path: PathBuf) -> usize {
        self.downloads
            .entry(path)
            .or_insert(AtomicUsize::new(0))
            .fetch_add(1, Ordering::SeqCst)
            + 1
    }
}

///Manages file system
#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FileManager {
    timestamps: DashMap<PathBuf, usize>,
    #[serde(skip_serializing)]
    base: PathBuf,
}

impl FileManager {
    pub fn new(path: PathBuf) -> Self {
        FileManager {
            timestamps: DashMap::new(),
            base: path,
        }
    }

    pub fn update_path_timestamp(&self, file: PathBuf) {
        self.timestamps.insert(
            file.strip_prefix(&self.base).unwrap().to_path_buf(),
            Utc::now().timestamp_millis() as usize,
        );
    }

    pub fn get_path_timestamp(&self, file: &PathBuf) -> Option<usize> {
        Some(
            *self
                .timestamps
                .get(&file.strip_prefix(&self.base).unwrap().to_path_buf())?,
        )
    }

    pub fn get_path_timestamp_raw(&self, file: &PathBuf) -> Option<usize> {
        Some(*self.timestamps.get(file)?)
    }
}
