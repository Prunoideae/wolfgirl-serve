mod api;
mod app;
mod responder;
mod routes;
mod stats;

use std::{fs, path::PathBuf, str::FromStr, sync::Arc};

use crate::routes::backend;
use crate::stats::DownloadCount;
use api::query;
use app::Args;
use clap::Parser;
use lazy_static::lazy_static;
use notify::{
    event::{CreateKind, DataChange, ModifyKind, RemoveKind},
    EventKind, RecommendedWatcher,
    RecursiveMode::Recursive,
    Watcher,
};
use rocket::{
    routes,
    shield::{NoSniff, Shield},
    Config,
};
use rocket_dyn_templates::Template;
use routes::{frontend, static_file};
use stats::FileManager;
use std::env;

lazy_static! {
    pub static ref THIS_DIR: PathBuf = env::current_dir().unwrap();
    pub static ref STATIC_DIR: PathBuf = THIS_DIR.join("static");
}

/// The implementation of static.wolfgirl.moe
/// -----------------------------------------
/// This domain is now served as a full-purpose static file server
/// with additional properties
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args = Args::parse();

    let base_dir = fs::canonicalize(args.dir.unwrap_or(PathBuf::from_str(".").unwrap())).unwrap();
    let manager = Arc::new(FileManager::new(base_dir.clone()));

    let watcher_manager = manager.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(CreateKind::File)
                    | EventKind::Modify(ModifyKind::Data(DataChange::Content))
                    | EventKind::Remove(RemoveKind::File) => event
                        .paths
                        .into_iter()
                        .for_each(|p| watcher_manager.update_path_timestamp(p)),
                    _ => (),
                };
            };
        },
        notify::Config::default(),
    )
    .unwrap();
    watcher.watch(&base_dir, Recursive).unwrap();

    let shield = Shield::default().disable::<NoSniff>();
    let _ = rocket::custom(
        Config::figment()
            .merge(("port", args.port))
            .merge(("address", args.addr.unwrap_or("127.0.0.1".to_string())))
            .merge(("workers", args.workers)),
    )
    .attach(shield)
    .attach(Template::fairing())
    .manage(base_dir)
    .manage(DownloadCount::new())
    .manage(manager)
    .mount(
        "/object-service",
        routes![backend::get_object, backend::get_object_checked],
    )
    .mount(
        "/api",
        routes![
            query::get_count,
            query::get_timestamps,
            query::get_timestamp
        ],
    )
    .mount("/drive", routes![frontend::serve_download])
    .mount("/static", routes![static_file::serve_static])
    .ignite()
    .await?
    .launch()
    .await?;
    Ok(())
}
