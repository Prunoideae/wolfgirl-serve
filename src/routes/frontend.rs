use std::path::PathBuf;

use ignore::WalkBuilder;
use rocket::{get, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::stats::DownloadCount;

#[derive(Debug, Serialize, PartialEq, PartialOrd, Ord, Eq)]
enum ObjectType {
    Dir,
    File,
}

#[derive(Debug, Serialize, PartialEq, PartialOrd, Ord, Eq)]
struct File {
    path_type: ObjectType,
    name: String,
    path: String,
    downloads: String,
}

#[derive(Debug, Serialize)]
struct Breadcrumb<'a> {
    name: &'a str,
    path: String,
}

#[get("/<path..>")]
pub async fn serve_download(
    path: PathBuf,
    count: &State<DownloadCount>,
    base_path: &State<PathBuf>,
) -> Option<Template> {
    let path_clone = path.clone();
    let path = base_path.join(path);

    if !path.exists() || path.is_file() {
        return None;
    };

    let base_breadcrumb = Breadcrumb {
        name: base_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default(),
        path: "/drive/".to_owned(),
    };
    let breadcrumbs = vec![base_breadcrumb]
        .into_iter()
        .chain(
            path.strip_prefix(base_path.inner())
                .unwrap()
                .iter()
                .map(|s| s.to_str().unwrap())
                .scan("".to_owned(), |path, name| {
                    path.push('/');
                    path.push_str(name);
                    Some(Breadcrumb {
                        name,
                        path: format!("/drive{}", path),
                    })
                }),
        )
        .collect::<Vec<_>>();

    let mut items = WalkBuilder::new(&path)
        .standard_filters(false)
        .git_ignore(true)
        .hidden(true)
        .max_depth(Some(1))
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| &path != entry.path())
        .filter(|entry| !entry.path_is_symlink())
        .map(|entry| {
            let abs_path = entry.path();
            let rel_path = abs_path.strip_prefix(base_path.inner()).unwrap();
            let path_ref = rel_path.to_str().unwrap_or_default();
            File {
                path_type: if abs_path.is_file() {
                    ObjectType::File
                } else {
                    ObjectType::Dir
                },
                name: rel_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_owned(),
                path: format!(
                    "/{}/{}",
                    if abs_path.is_file() {
                        "object-service/unchecked"
                    } else {
                        "drive"
                    },
                    path_ref.to_string()
                ),
                downloads: if abs_path.is_file() {
                    count.get_count(&rel_path.to_path_buf()).to_string()
                } else {
                    "-".to_owned()
                },
            }
        })
        .collect::<Vec<_>>();

    if &path != base_path.inner() {
        items.push(File {
            path_type: ObjectType::Dir,
            name: "..".to_owned(),
            path: format!(
                "/drive/{}",
                path_clone.parent().unwrap().to_str().unwrap_or_default()
            ),
            downloads: "-".to_owned(),
        })
    }

    items.sort_unstable();

    Some(Template::render(
        "drive",
        context! {
            dir_name: path.file_name().and_then(|s|s.to_str()).unwrap_or_default().to_owned(),
            files: items,
            breadcrumbs,
            style: include_str!("style.css")
        },
    ))
}
