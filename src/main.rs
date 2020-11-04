use std::io;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use crate::clip::SentryClip;
use chrono::{Duration, Utc};
use simplelog::{ConfigBuilder, TermLogger, TerminalMode};
use structopt::StructOpt;

mod camera;
mod clip;
mod formats;

// CLI options
#[derive(Clone, Debug, StructOpt)]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long)]
    ffmpeg_location: PathBuf,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbosity: u8,

    /// root folder containing the sentry clips
    #[structopt(short, long)]
    clips_path: PathBuf,
}

fn setup_logging(c: &Opt) {
    let level = match c.verbosity {
        0 => log::LevelFilter::Info, //default
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let logger_config = ConfigBuilder::new()
        .set_time_to_local(true)
        // The default doesn't include the date which can get confusing
        // This sets it to yyyy-mm-dd hh:mm:ss
        .set_time_format_str("%F %H:%M:%S")
        .build();
    let _ = TermLogger::init(level, logger_config, TerminalMode::Mixed);
}

fn main() -> io::Result<()> {
    let cli_options = Opt::from_args();
    setup_logging(&cli_options);
    let sentryclips_root = Path::new(&cli_options.clips_path);
    let clips_folders = children_folders(sentryclips_root)?;
    for dir_entry in clips_folders {
        match SentryClip::from_folder(&dir_entry, &cli_options.ffmpeg_location) {
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
