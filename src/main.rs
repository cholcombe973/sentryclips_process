use std::path::Path;
use crate::camera::CameraFile;
use std::io;
use std::borrow::Borrow;
use std::fs::DirEntry;

mod camera;

fn main() {
    println!("Hello, world!");
}

fn process_folder(root: &Path) -> io::Result<Vec<CameraFile>> {
    Ok(
        list_files(root)?.into_iter().filter_map(|e| CameraFile::from(e.path().as_path()).ok()).collect()
    )
}

fn list_files(root: &Path) -> io::Result<Vec<DirEntry>> { //TODO Filter files
    Ok(root.read_dir()?.filter_map(Result::ok).collect())
}
