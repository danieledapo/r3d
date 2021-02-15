pub mod obj;
pub mod stl;

use std::{
    fs::File,
    io::{self, BufReader},
    num,
    path::Path,
};

use crate::Vec3;

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

/// Base features an input mesh must have in order to be rendered.
///
/// As of now, it's very basic and all that's required is that it provides a
/// list of triangles composing the mesh.
pub trait Mesh {
    /// Return all the triangular loops of the mesh.
    ///
    /// Any non triangular loops are skipped.
    fn triangles(&self) -> Box<dyn Iterator<Item = [Vec3; 3]> + '_>;
}

/// Load the mesh at `path` trying to guess the format by the file extension.
///
/// STL and OBJ are the only supported formats as of now.
pub fn load_mesh(path: impl AsRef<Path>) -> Result<Box<dyn Mesh>> {
    let ext = path.as_ref().extension().ok_or(Error::BadFormat)?;

    if ext == "obj" {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        return Ok(Box::new(obj::Obj::load(reader)?));
    }

    if ext == "stl" {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        return Ok(Box::new(stl::Stl::load(reader)?));
    }

    Err(Error::BadFormat)
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
