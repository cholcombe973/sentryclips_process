extern crate chrono;

use std::path::PathBuf;
use self::chrono::{NaiveDateTime, Utc};
use crate::camera::{CameraFile, Camera};
use std::fs::{DirEntry, File, rename};
use std::io;

use crate::formats::{parse_tesla_timestamp, parse_error_to_io_error, file_stem, err_from_str};
use std::process::Command;
use std::io::Write;

pub struct SentryClip {
    pub folder: PathBuf,
    pub when: NaiveDateTime,
    pub clips: Vec<CameraFile>,
}

impl SentryClip {
    pub fn from_folder(entry: &DirEntry) -> io::Result<SentryClip> {
        let clips = process_folder(entry)?;
        let when = parse_tesla_timestamp(file_stem(entry.path().as_path())?.as_str()).map_err(parse_error_to_io_error)?;
        let folder = entry.path();
        Ok(SentryClip { folder, when, clips })
    }

    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }

    pub fn files_per_camera(&self, camera: &Camera) -> Vec<&CameraFile> {
        let mut res = self.clips.iter().filter(|f| f.camera.eq(camera)).collect::<Vec<&CameraFile>>();
        res.sort_by(|a,b| a.start_time.cmp(&b.start_time));
        res
    }

    pub fn concatenate_camera_files(&self, camera: &Camera) -> io::Result<String> {
        let files = &self.files_per_camera(&camera);
        let result_file  = self.folder.join(format!("{}-{}.mp4", self.when.format("%Y-%m-%d_%H-%M-%S"), &camera.camera_file_name()));
        log::info!("Attaching files {:?} into file {}", files.iter().map(move |f| f.path.display().to_string()).collect::<Vec<String>>(), result_file.display());
        let date_format = "%Y%m%d_%H%M%S%3f";
        let now = Utc::now().format(date_format);
        let when = self.when.format(date_format);
        let playlist_filename = format!("/tmp/tesla_playlist_tmp_{}_{}_{}.txt", now, when ,camera.camera_file_name());
        create_temp_playlist(files, playlist_filename.as_str())?;
        let result_tmp_file_path = self.folder.join(
            format!("{}-tmp.mp4", result_file.file_stem().and_then(|f| f.to_str()).ok_or(err_from_str(format!("Cannot get file name for file {}", result_file.display()).as_str()))?)
        );
        let result_tmp_file= result_tmp_file_path.to_str().ok_or(err_from_str("Cannot build a path for temporary file"))?;
        let _status = Command::new("ffmpeg")
            .args(&["-f", "concat", "-safe", "0", "-i", playlist_filename.as_str(), "-c", "copy", result_tmp_file])
            .status()?;
        Ok(result_tmp_file.to_string())
    }

}

fn create_temp_playlist(files: &Vec<&CameraFile>, playlist_filename: &str) -> io::Result<()> {
    let mut file = File::create(playlist_filename)?;
    for c in files.iter() {
        let row = format!("file '{}'", c.path.display());
        file.write(row.as_bytes())?;
        file.write("\n".as_bytes())?;
    }
    file.sync_all()
}

fn process_folder(root: &DirEntry) -> io::Result<Vec<CameraFile>> {
    let clips: Vec<CameraFile> = list_files(root)?.into_iter().filter_map(|e| {
        match CameraFile::from(e.path().as_path()) {
            Ok(f) => { Some(f) },
            Err(err) => { log::error!("Found error processing clip {}: {}", &e.path().display(), err); None },
        }
    }).collect();
    log::debug!("Processed {} sentry clips", &clips.len());
    Ok(clips)
}

fn list_files(root: &DirEntry) -> io::Result<Vec<DirEntry>> {
    log::debug!("Finding files in folder {}", root.path().display());
    let children: Vec<DirEntry> = root.path().read_dir()?.filter_map(|res| {
        match res {
            Ok(e) => {
                log::info!("Found child {}", e.path().display());
                if e.path().is_file() { Some(e) } else { None }
            }
            Err(err) => {
                log::error!("Found error: {}", err);
                None
            }
        }
    }).collect();
    log::info!("Found {} clip files in folder {}", &children.len(), &root.path().display());
    Ok(children)
}