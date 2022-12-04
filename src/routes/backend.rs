use std::{fs, path::PathBuf, sync::Arc};

use rocket::{fs::NamedFile, get, http::Status, State};

use crate::stats::{DownloadCount, FileManager};

#[get("/unchecked/<path..>")]
pub async fn get_object(
    path: PathBuf,
    count: &State<DownloadCount>,
    base_path: &State<PathBuf>,
) -> Option<NamedFile> {
    let file_path = base_path.join(path.clone());
    if !file_path.exists() || file_path.is_dir() {
        return None;
    }
    count.incr_count(path);
    NamedFile::open(file_path).await.ok()
}

#[get("/checked/<path..>?<timestamp>")]
pub async fn get_object_checked(
    path: PathBuf,
    count: &State<DownloadCount>,
    manager: &State<Arc<FileManager>>,
    base_path: &State<PathBuf>,
    timestamp: usize,
) -> Result<NamedFile, (Status, &'static str)> {
    let path = base_path.join(path);
    if !path.exists() || path.is_dir() {
        return Err((Status::NotFound, "Not found"));
    }
    let path = fs::canonicalize(path).unwrap();

    match manager.get_path_timestamp(&path) {
        Some(file_stamp) => {
            if file_stamp == timestamp {
                Err((Status::NotModified, "Content not modified"))
            } else {
                count.incr_count(path.clone());
                NamedFile::open(path)
                    .await
                    .map_err(|_| (Status::InternalServerError, "Errored!"))
            }
        }
        None => {
            manager.update_path_timestamp(path.clone());
            count.incr_count(path.clone());
            NamedFile::open(path)
                .await
                .map_err(|_| (Status::InternalServerError, "Errored!"))
        }
    }
}
