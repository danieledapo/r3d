use std::convert::{From, TryFrom};
use std::io;
use std::io::BufRead;
use std::num;

use crate::Vec3;

/// Obj mesh read from an obj file.
///
/// Currently only a subset of the format is supported that is
/// only vertices and loops are read. Other features like vertex normals,
/// texture coordinates and groups are not supported.
pub struct Obj {
    comments: Vec<String>,
    vertices: Vec<Vec3>,
    loops: Vec<Vec<isize>>,
}

/// Result type returned by `load_obj`.
pub type Result<T> = std::result::Result<T, Error>;

/// Possible errors returned by `load_obj`.
#[derive(Debug)]
pub enum Error {
    /// The obj file was malformed or truncated, therefore it was not possible
    /// to decode it.
    BadFormat,

    /// Error while parsing a number.
    InvalidNumber,

    /// IO error.
    IoError(io::Error),
}

/// Try to load an `ObjMesh` from the given reader.
pub fn load_obj(r: impl BufRead) -> Result<Obj> {
    let mut mesh = Obj {
        comments: vec![],
        vertices: vec![],
        loops: vec![],
    };

    for l in r.lines() {
        let l = l?;

        let mut tokens = l.split_whitespace();

        let id = tokens.next().ok_or(Error::BadFormat)?;

        match id {
            "#" => {
                mesh.comments.push(l.clone());
            }
            "v" => {
                let x = tokens.next().ok_or(Error::BadFormat)?.parse()?;
                let y = tokens.next().ok_or(Error::BadFormat)?.parse()?;
                let z = tokens.next().ok_or(Error::BadFormat)?.parse()?;

                mesh.vertices.push(Vec3::new(x, y, z));
            }
            "f" => {
                let l = tokens
                    .map(|t| {
                        // ignore all vertex info, but the vertex index itself
                        let mut toks = t.split('/');
                        let vi = toks.next().ok_or(Error::BadFormat)?.parse()?;

                        // obj is 1-based
                        if vi == 0 {
                            return Err(Error::BadFormat);
                        }

                        Ok(vi)
                    })
                    .collect::<Result<Vec<isize>>>();
                let l = l?;

                mesh.loops.push(l);
            }
            "vn" | "vp" | "vt" | "s" => {
                // not supported
            }
            _ => return Err(Error::BadFormat),
        }
    }

    Ok(mesh)
}

impl Obj {
    /// Return all the triangular loops of the mesh.
    ///
    /// Any non triangular loops are skipped.
    pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.loops.iter().filter_map(move |l| {
            if l.len() != 3 {
                return None;
            }

            let get_v = |i: isize| {
                self.vertices[if i > 0 {
                    usize::try_from(i - 1).unwrap()
                } else {
                    self.vertices.len() - usize::try_from(i.abs()).unwrap()
                }]
            };

            Some([get_v(l[0]), get_v(l[1]), get_v(l[2])])
        })
    }
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
