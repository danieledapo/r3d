use std::ffi::OsStr;
use std::io;
use std::process::Command;

pub type Result<R> = std::result::Result<R, OpenError>;

#[derive(Debug)]
pub enum OpenError {
    IoError(io::Error),
    OpenerError,
}

#[cfg(target_os = "linux")]
const OPENER: &str = "xdg-open";

#[cfg(target_os = "macos")]
const OPENER: &str = "open";

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn open(path: impl AsRef<OsStr>) -> Result<()> {
    let exit = Command::new(OPENER).arg(path.as_ref()).spawn()?.wait()?;

    if exit.success() {
        Ok(())
    } else {
        Err(OpenError::OpenerError)
    }
}

#[cfg(target_os = "windows")]
pub fn open(path: impl AsRef<OsStr>) -> Result<()> {
    // meh, windows will come maybe :)
    Ok(())
}

impl From<io::Error> for OpenError {
    fn from(e: io::Error) -> Self {
        OpenError::IoError(e)
    }
}
