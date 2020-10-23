use std::env::args;
use std::fs::DirEntry;
use std::io;
use std::path::Path;

use crate::clip::SentryClip;
use crate::formats::err_from_str;
use chrono::{Duration, Utc};

mod camera;
mod clip;
mod formats;

fn main() -> io::Result<()> {
    env_logger::init();
    let arguments: Vec<String> = args().collect();
    let path = arguments.get(1).ok_or(err_from_str(
        "Need an argument indicating the root folder containing the sentry clips",
    ))?;
    let sentryclips_root = Path::new(path);
    let clips_folders = children_folders(sentryclips_root)?;
    for dir_entry in clips_folders {
        match SentryClip::from_folder(&dir_entry) {
            Err(e) => log::warn!(
                "Cannot process folder {} as a Sentry Clip folder: {}",
                dir_entry.path().display(),
                e
            ),
            Ok(clip) if clip.is_empty() => log::info!(
                "Not processing folder {} because it does not contain any clips",
                clip.folder.display()
            ),
            Ok(clip) if clip.last_modified + Duration::minutes(15) > Utc::now().naive_utc() => {
                log::info!(
                    "Not processing folder {}, it is too recent",
                    clip.folder.display()
                )
            }
            Ok(clip) => clip.process(),
        }
    }

    Ok(())
}

fn children_folders(path: &Path) -> io::Result<Vec<DirEntry>> {
    assert!(path.exists() && path.is_dir());
    let mut folders: Vec<DirEntry> = path
        .read_dir()?
        .filter_map(|f| {
            f.ok()
                .and_then(|e| if e.path().is_dir() { Some(e) } else { None })
        })
        .collect();
    folders.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(folders)
}
