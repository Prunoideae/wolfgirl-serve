use std::{path::PathBuf, sync::Arc};

use path_clean::PathClean;
use rocket::{get, serde::json::Json, State};

use crate::stats::{DownloadCount, FileManager};

#[get("/downloads")]
pub async fn get_count(download_count: &State<DownloadCount>) -> Json<&DownloadCount> {
    Json(&download_count)
}

#[get("/timestamps")]
pub async fn get_timestamps(manager: &State<Arc<FileManager>>) -> Json<&FileManager> {
    Json(&manager)
}

#[get("/timestamp?<path>")]
pub async fn get_timestamp(manager: &State<Arc<FileManager>>, path: String) -> Option<String> {
    manager
        .get_path_timestamp_raw(&PathBuf::from(path).clean())
        .map(|x| x.to_string())
}
