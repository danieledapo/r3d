//! This module contains functions to load [binary and ascii STL].

use std::{
    convert::TryFrom,
    io::{BufRead, Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt};

use super::{Error, Mesh, Result};
use crate::{v3, Triangle, Vec3};

/// A STL format type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StlFormat {
    Ascii,
    Binary,
}

/// STL mesh read from a STL file.
///
/// Both binary and ASCII STL files are supported.
pub struct Stl {
    header: Vec<u8>,
    triangles: Vec<StlTriangle>,
}

/// A STL triangle.
#[derive(Debug, PartialEq)]
pub struct StlTriangle {
    pub triangle: Triangle,
    pub normal: Vec3,
    pub attributes: Vec<u8>,
}

impl Stl {
    /// Return the header of the STL file that is either the comment or the name
    /// of the object.
    pub fn header(&self) -> &[u8] {
        &self.header
    }

    /// Try to load a `Stl` mesh from the given reader.
    ///
    /// The mesh format is automatically guessed from the contents of the
    /// reader.
    pub fn load(mut r: impl BufRead + Seek) -> Result<Self> {
        let format = guess_stl_format(&mut r)?;

        match format {
            StlFormat::Ascii => {
                let mut content = String::new();
                r.read_to_string(&mut content)?;

                let (header, triangles) = load_ascii_stl(&content)?;

                Ok(Self {
                    header: header.as_bytes().to_vec(),
                    triangles,
                })
            }
            StlFormat::Binary => {
                let (header, triangles) = load_binary_stl(r)?;
                Ok(Self {
                    header: header.to_vec(),
                    triangles,
                })
            }
        }
    }
}

impl Mesh for Stl {
    fn triangles(&self) -> Box<dyn Iterator<Item = Triangle> + '_> {
        Box::new(self.triangles.iter().map(|t| t.triangle.clone()))
    }
}

/// Try to guess the format of a STL by looking at the first bytes. If they're
/// "solid" then it's probably an ASCII STL, binary otherwise. The reader's
/// position is reset to the beginning of the buffer.
pub fn guess_stl_format<R: BufRead + Seek>(r: &mut R) -> Result<StlFormat> {
    let mut header = [0; 5];
    r.read_exact(&mut header)?;

    let format = match &header {
        b"solid" => StlFormat::Ascii,
        _ => StlFormat::Binary,
    };

    r.seek(SeekFrom::Start(0))?;

    Ok(format)
}

/// Load a [binary STL][0] from a given reader. Returns a tuple composed of the
/// STL header and an iterator over the triangles of the STL.
///
/// [0]: https://en.wikipedia.org/wiki/STL_(file_format)#Binary_STL
pub fn load_binary_stl<R: BufRead>(mut r: R) -> Result<([u8; 80], Vec<StlTriangle>)> {
    let mut header = [0; 80];
    r.read_exact(&mut header)?;

    let read_vec3 = |r: &mut R| -> Result<Vec3> {
        let x = r.read_f32::<LittleEndian>()?;
        let y = r.read_f32::<LittleEndian>()?;
        let z = r.read_f32::<LittleEndian>()?;

        Ok(v3(f64::from(x), f64::from(y), f64::from(z)))
    };

    let ntriangles = r.read_u32::<LittleEndian>()?;

    let mut triangles = Vec::with_capacity(usize::try_from(ntriangles).unwrap_or(0));
    for _ in 0..ntriangles {
        let normal = read_vec3(&mut r)?;
        let v0 = read_vec3(&mut r)?;
        let v1 = read_vec3(&mut r)?;
        let v2 = read_vec3(&mut r)?;

        let attribute_count = r.read_u16::<LittleEndian>()?;

        let mut attributes = vec![0; usize::from(attribute_count)];
        r.read_exact(&mut attributes)?;

        triangles.push(StlTriangle {
            attributes,
            normal,
            triangle: Triangle::new(v0, v1, v2),
        });
    }

    Ok((header, triangles))
}

/// Load a [ASCII STL][0] from a given string. Return a tuple composed of the
/// solid name and the triangles of the STL.
///
/// [0]: https://en.wikipedia.org/wiki/STL_(file_format)#ASCII_STL
pub fn load_ascii_stl(stl: &str) -> Result<(&str, Vec<StlTriangle>)> {
    let mut tokens = stl.split_whitespace().peekable();

    if tokens.next() != Some("solid") {
        return Err(Error::BadFormat);
    }

    let name = match tokens.peek() {
        None => return Err(Error::BadFormat),
        Some(&"facet") => "",
        Some(_name) => tokens.next().unwrap(),
    };

    let mut tris = vec![];

    loop {
        match tokens.next() {
            Some("endsolid") => break,
            Some("facet") => (),
            _ => return Err(Error::BadFormat),
        };

        if tokens.next() != Some("normal") {
            return Err(Error::BadFormat);
        }

        let nx: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;
        let ny: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;
        let nz: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;

        if tokens.next() != Some("outer") {
            return Err(Error::BadFormat);
        }

        if tokens.next() != Some("loop") {
            return Err(Error::BadFormat);
        }

        let mut vs = Vec::with_capacity(3);
        loop {
            match tokens.next() {
                Some("endloop") => break,
                Some("vertex") => (),
                _ => return Err(Error::BadFormat),
            }

            let vx: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;
            let vy: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;
            let vz: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;

            vs.push(v3(vx, vy, vz));
        }

        if vs.len() != 3 {
            return Err(Error::BadFormat);
        }

        tris.push(StlTriangle {
            normal: v3(nx, ny, nz),
            triangle: Triangle::new(vs[0], vs[1], vs[2]),
            attributes: vec![],
        });

        if tokens.next() != Some("endfacet") {
            return Err(Error::BadFormat);
        }
    }

    Ok((name, tris))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{BufReader, Cursor};

    #[test]
    fn test_load_binary_stl() {
        let cube_stl = include_bytes!("../../../data/cube.stl");
        let mut r = BufReader::new(Cursor::new(&cube_stl[..]));

        assert_eq!(guess_stl_format(&mut r).unwrap(), StlFormat::Binary);

        let (header, tris) = load_binary_stl(r).unwrap();

        assert_eq!(
            &header[..],
            &[
                b'E', b'x', b'p', b'o', b'r', b't', b'e', b'd', b' ', b'f', b'r', b'o', b'm', b' ',
                b'B', b'l', b'e', b'n', b'd', b'e', b'r', b'-', b'2', b'.', b'7', b'9', b' ', b'(',
                b's', b'u', b'b', b' ', b'0', b')', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0,
            ][..]
        );

        assert_eq!(
            tris,
            vec![
                StlTriangle {
                    attributes: vec![],
                    normal: v3(-1.0, 0.0, 0.0),
                    triangle: Triangle::new(
                        v3(-1.0, -1.0, -1.0),
                        v3(-1.0, -1.0, 1.0),
                        v3(-1.0, 1.0, 1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(-1.0, 0.0, 0.0),
                    triangle: Triangle::new(
                        v3(-1.0, 1.0, 1.0),
                        v3(-1.0, 1.0, -1.0),
                        v3(-1.0, -1.0, -1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0, 1, 0),
                    triangle: Triangle::new(v3(-1.0, 1.0, -1.0), v3(-1.0, 1.0, 1.0), v3(1, 1, 1),),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0, 1, 0),
                    triangle: Triangle::new(v3(1, 1, 1), v3(1.0, 1.0, -1.0), v3(-1.0, 1.0, -1.0),),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(1, 0, 0),
                    triangle: Triangle::new(v3(1.0, 1.0, -1.0), v3(1, 1, 1), v3(1.0, -1.0, 1.0),),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(1, 0, 0),
                    triangle: Triangle::new(
                        v3(1.0, -1.0, 1.0),
                        v3(1.0, -1.0, -1.0),
                        v3(1.0, 1.0, -1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.0, -1.0, 0.0),
                    triangle: Triangle::new(
                        v3(-1.0, -1.0, 1.0),
                        v3(-1.0, -1.0, -1.0),
                        v3(1.0, -1.0, -1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.0, -1.0, 0.0),
                    triangle: Triangle::new(
                        v3(1.0, -1.0, -1.0),
                        v3(1.0, -1.0, 1.0),
                        v3(-1.0, -1.0, 1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.0, 0.0, -1.0),
                    triangle: Triangle::new(
                        v3(1.0, -1.0, -1.0),
                        v3(-1.0, -1.0, -1.0),
                        v3(-1.0, 1.0, -1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.0, 0.0, -1.0),
                    triangle: Triangle::new(
                        v3(-1.0, 1.0, -1.0),
                        v3(1.0, 1.0, -1.0),
                        v3(1.0, -1.0, -1.0),
                    ),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0, 0, 1),
                    triangle: Triangle::new(v3(1, 1, 1), v3(-1.0, 1.0, 1.0), v3(-1.0, -1.0, 1.0),),
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0, 0, 1),
                    triangle: Triangle::new(v3(-1.0, -1.0, 1.0), v3(1.0, -1.0, 1.0), v3(1, 1, 1),),
                },
            ]
        );
    }

    #[test]
    fn test_load_ascii_stl() {
        let stl = r"solid cube_corner
            facet normal 0.0 -1.0 0.0
                outer loop
                    vertex 0.0 0.0 0.0
                    vertex 1.0 0.0 0.0
                    vertex 0.0 0.0 1.0
                endloop
            endfacet
            facet normal 0.0 0.0 -1.0e0
                outer loop
                    vertex 0.0 0.0 0.0
                    vertex 0.0 1.0 0.0
                    vertex 1.0 0.0 0.0
                endloop
            endfacet
            facet normal -1.0 0.0 0.0
                outer loop
                    vertex 0.0 0.0 0.0
                    vertex 0.0 0.0 1.0
                    vertex 0.0 1.0 0.0
                endloop
            endfacet
            facet normal 0.577 0.577 5.77e-1
                outer loop
                    vertex 1.0 0.0 0.0
                    vertex 0.0 1.0 0.0
                    vertex 0.0 0.0 1.0
                endloop
            endfacet
            endsolid
        ";

        assert_eq!(
            guess_stl_format(&mut BufReader::new(Cursor::new(stl))).unwrap(),
            StlFormat::Ascii
        );

        let (name, tris) = load_ascii_stl(stl).unwrap();

        assert_eq!(name, "cube_corner");
        assert_eq!(
            tris,
            vec![
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.0, -1.0, 0.0),
                    triangle: Triangle::new(Vec3::zero(), v3(1, 0, 0), v3(0, 0, 1))
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.0, 0.0, -1.0),
                    triangle: Triangle::new(Vec3::zero(), v3(0, 1, 0), v3(1, 0, 0))
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(-1.0, 0.0, 0.0),
                    triangle: Triangle::new(Vec3::zero(), v3(0, 0, 1), v3(0, 1, 0))
                },
                StlTriangle {
                    attributes: vec![],
                    normal: v3(0.577, 0.577, 0.577),
                    triangle: Triangle::new(v3(1, 0, 0), v3(0, 1, 0), v3(0, 0, 1))
                }
            ]
        )
    }
}
