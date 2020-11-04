use chrono::{NaiveDateTime, ParseError, ParseResult};
use std::io;
use std::path::Path;

pub fn parse_error_to_io_error(err: ParseError) -> io::Error {
    err_from_str(format!("{}", err).as_str())
}

pub fn err_from_str(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

pub fn parse_tesla_timestamp(date_str: &str) -> ParseResult<NaiveDateTime> {
    //2020-10-22_10-37-28-right_repeater
    let res = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d_%H-%M-%S");
    res.err()
        .map(|err| log::error!("Error parsing time '{}': {}", date_str, err));
    res
}

pub fn file_stem(path: &Path) -> io::Result<String> {
    path.file_stem()
        .and_then(|f| f.to_str().map(|s| s.to_owned()))
        .ok_or(err_from_str("Invalid filename"))
}
