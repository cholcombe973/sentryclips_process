use std::env::args;
use std::fs::DirEntry;
use std::io;
use std::path::Path;

use crate::formats::err_from_str;
use crate::clip::SentryClip;

mod camera;
mod clip;
mod formats;

use camera::Camera;

fn main() -> io::Result<()> {
    env_logger::init();
    let arguments: Vec<String> = args().collect();
    let path = arguments.get(1).ok_or(err_from_str("Need an argument indicating the root folder containing the sentry clips"))?;
    let sentryclips_root = Path::new(path);
    let clips_folders = children_folders(sentryclips_root)?;
    let clips: Vec<SentryClip> = clips_folders.into_iter().filter_map(|e|SentryClip::from_folder(&e).ok()).collect();

    let _clips_and_cameras: Vec<(SentryClip, Vec<(String, Camera)>)> = clips.into_iter().filter_map(|clip| {
        if !clip.is_empty() {
            log::info!("Processing clip folder {} [{}]", &clip.folder.display(), &clip.when);
            let all_cameras = Camera::all_cameras().into_iter();
            let clip_files_and_camera: Vec<(String, Camera)> = all_cameras.filter_map(|camera| {
                clip.concatenate_camera_files(&camera).ok().map(|f| (f, camera))
            }).collect();
            //Create mosaic
            clip.create_mosaic(&clip_files_and_camera)
                .map_err(|e| log::error!("Error creating mosaic for clip {}: {}", clip.folder.display(), e)).unwrap();
            Some((clip, clip_files_and_camera))
        } else { None}
    }).collect();

    Ok(())
}

fn children_folders(path: &Path) -> io::Result<Vec<DirEntry>> {
    assert!(path.exists() && path.is_dir());
    Ok(
        path.read_dir()?.filter_map(|f| {
            f.ok().and_then(|e| if e.path().is_dir() { Some(e) } else { None })
        }).collect()
    )
}
