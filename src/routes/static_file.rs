use std::path::PathBuf;

use rocket::{fs::NamedFile, get};

use crate::STATIC_DIR;

#[get("/<path..>")]
pub async fn serve_static(path: PathBuf) -> Option<NamedFile> {
    let path = STATIC_DIR.join(path);
    if !path.exists() || path.is_dir() {
        return None;
    }
    NamedFile::open(path).await.ok()
}
