pub mod obj;
pub mod stl;

use std::{io, num};

/// Result type returned by `Mesh::load`.
pub type Result<T> = std::result::Result<T, Error>;

/// Possible errors while loading a mesh.
#[derive(Debug)]
pub enum Error {
    /// The mesh file was malformed or truncated, therefore it was not possible
    /// to decode it.
    BadFormat,

    /// Error while parsing a number.
    InvalidNumber,

    /// IO error.
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(_e: num::ParseFloatError) -> Self {
        Error::InvalidNumber
    }
}

impl From<num::ParseIntError> for Error {
    fn from(_e: num::ParseIntError) -> Self {
        Error::InvalidNumber
    }
}
