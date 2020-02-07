extern crate chrono;
extern crate regex;

use std::io;
use std::path::{Path, PathBuf};

use regex::Regex;
use std::fmt::{Display, Formatter, Error};
use self::chrono::NaiveDateTime;
use crate::formats::{err_from_str, parse_error_to_io_error, parse_tesla_timestamp};

#[derive(PartialEq)]
pub enum Camera {
    Back,
    Front,
    Right,
    Left,
}

impl Display for Camera {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Camera::Back => f.write_str("Back"),
            Camera::Front => f.write_str("Front"),
            Camera::Left => f.write_str("Left"),
            Camera::Right => f.write_str("Right"),
        }
    }
}

impl Camera {
    pub fn all_cameras() -> Vec<Camera> {
        vec![Camera::Back, Camera::Front, Camera::Left, Camera::Right]
    }

    pub fn from(name: &str) -> Option<Camera> {
        Camera::all_cameras().into_iter().find(|c| c.camera_file_name().eq(name))
    }

    pub fn camera_file_name(&self) -> &str {
        match self {
            Camera::Back => "back",
            Camera::Front => "front",
            Camera::Right => "right_repeater",
            Camera::Left => "left_repeater"
        }
    }

}

pub struct CameraFile {
    pub path: PathBuf,
    pub camera: Camera,
    pub start_time: NaiveDateTime,
}

impl Display for CameraFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} - {} - {}", self.path.display(), self.start_time, self.camera)
    }
}

impl CameraFile {
    pub fn from(path: &Path) -> io::Result<CameraFile> {
        assert!(path.extension().and_then(|f| f.to_str()) == Some("mp4"));
        let file_name = path.file_stem().and_then(|f| f.to_str()).ok_or(err_from_str("Invalid filename"))?;
        log::debug!("Creating a CamaraFile for file '{}'", file_name);
        let re = Regex::new(r"(\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2})-([a-zA-Z0-9_]+)").unwrap();
        let mut captures = re.captures_iter(file_name);
        let capture = captures.next().ok_or(err_from_str("Cannot identify the format of the file"))?;
        log::debug!("Identified time of capture ({}) and camera ({})", &capture[1], &capture[2]);
        let date_time = parse_tesla_timestamp(&capture[1]).map_err(parse_error_to_io_error)?;
        let camera = Camera::from(&capture[2]).ok_or(err_from_str("Cannot identify camera from the file name"))?;
        Ok(CameraFile { path: path.to_owned(), camera, start_time: date_time })
    }
}