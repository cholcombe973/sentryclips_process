extern crate chrono;
extern crate regex;

use std::fs::DirEntry;
use std::io;
use std::io::Error;
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset, ParseError, ParseResult};

use regex::Regex;

enum Camera {
    Back,
    Front,
    Right,
    Left,
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

    fn parse_date(date_str: &str) -> ParseResult<DateTime<FixedOffset>> {
        DateTime::parse_from_str(date_str, "&Y-%m-%d_%H-%M-%S")
    }
}

fn parse_error_to_io_error(err: ParseError) -> io::Error {
    err_from_str(format!("{}", err).as_str())
}

fn err_from_str(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

pub struct CameraFile {
    path: PathBuf,
    camera: Camera,
    start_time: DateTime<FixedOffset>,
}

impl CameraFile {
    pub fn from(path: &Path) -> io::Result<CameraFile> {
        assert!(path.extension().and_then(|f| f.to_str()) == Some("mp4"));
        let file_name = path.file_stem().and_then(|f| f.to_str()).ok_or(err_from_str("Invalid filename"))?;
        let re = Regex::new(r"(\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2})-([a-zA-Z0-9_]+)").unwrap();
        let mut captures = re.captures_iter(file_name);
        let capture = captures.next().ok_or(err_from_str("Cannot identify the format of the file"))?;
        let date_time = Camera::parse_date(&capture[1]).map_err(parse_error_to_io_error)?;
        let camera = Camera::from(&capture[2]).ok_or(err_from_str("Cannot identify camera from the file name"))?;
        Ok(CameraFile { path: path.to_owned(), camera, start_time: date_time })
    }
}