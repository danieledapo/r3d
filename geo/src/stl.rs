//! This module contains functions to load [binary and ascii STL].

use std::io::{BufRead, Result, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::vec3::Vec3;

/// A STL format type.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StlFormat {
    Ascii,
    Binary,
}

/// A STL triangle.
#[derive(Debug, PartialEq)]
pub struct StlTriangle {
    pub positions: [Vec3; 3],
    pub normal: Vec3,
    pub attributes: Vec<u8>,
}

/// An `Iterator` over a binary STL.
#[derive(Debug)]
pub struct BinaryStlIter<R> {
    ntriangles: u32,
    input: R,
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
pub fn load_binary_stl<R: BufRead>(mut r: R) -> Result<([u8; 80], BinaryStlIter<R>)> {
    let mut header = [0; 80];
    r.read_exact(&mut header)?;

    let ntriangles = r.read_u32::<LittleEndian>()?;

    Ok((
        header,
        BinaryStlIter {
            ntriangles,
            input: r,
        },
    ))
}

/// Load a [ASCII STL][0] from a given string. Return a tuple composed of the
/// solid name and the triangles of the STL.
///
/// [0]: https://en.wikipedia.org/wiki/STL_(file_format)#ASCII_STL
pub fn load_ascii_stl(stl: &str) -> Option<(&str, Vec<StlTriangle>)> {
    let mut tokens = stl.split_whitespace().peekable();

    if tokens.next()? != "solid" {
        return None;
    }

    let name = if *tokens.peek()? != "facet" {
        tokens.next().unwrap()
    } else {
        ""
    };

    let mut tris = vec![];

    loop {
        match tokens.next()? {
            "endsolid" => break,
            "facet" => (),
            _ => return None,
        };

        if tokens.next()? != "normal" {
            return None;
        }

        let nx: f64 = tokens.next()?.parse().ok()?;
        let ny: f64 = tokens.next()?.parse().ok()?;
        let nz: f64 = tokens.next()?.parse().ok()?;

        if tokens.next()? != "outer" {
            return None;
        }

        if tokens.next()? != "loop" {
            return None;
        }

        let mut vs = Vec::with_capacity(3);
        loop {
            match tokens.next()? {
                "endloop" => break,
                "vertex" => (),
                _ => return None,
            }

            let vx: f64 = tokens.next()?.parse().ok()?;
            let vy: f64 = tokens.next()?.parse().ok()?;
            let vz: f64 = tokens.next()?.parse().ok()?;

            vs.push(Vec3::new(vx, vy, vz));
        }

        if vs.len() != 3 {
            return None;
        }

        tris.push(StlTriangle {
            normal: Vec3::new(nx, ny, nz),
            positions: [vs[0], vs[1], vs[2]],
            attributes: vec![],
        });

        if tokens.next()? != "endfacet" {
            return None;
        }
    }

    Some((name, tris))
}

impl<R> Iterator for BinaryStlIter<R>
where
    R: BufRead,
{
    type Item = Result<StlTriangle>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ntriangles == 0 {
            return None;
        }

        self.ntriangles -= 1;

        Some(self.read_tri())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if u64::from(self.ntriangles) <= usize::max_value() as u64 {
            (self.ntriangles as usize, Some(self.ntriangles as usize))
        } else {
            (0, None)
        }
    }
}

impl<R> BinaryStlIter<R>
where
    R: BufRead,
{
    fn read_tri(&mut self) -> Result<StlTriangle> {
        let normal = self.read_vec3()?;
        let v0 = self.read_vec3()?;
        let v1 = self.read_vec3()?;
        let v2 = self.read_vec3()?;

        let attribute_count = self.input.read_u16::<LittleEndian>()?;

        let mut attributes = vec![0; usize::from(attribute_count)];
        self.input.read_exact(&mut attributes)?;

        Ok(StlTriangle {
            attributes,
            normal,
            positions: [v0, v1, v2],
        })
    }

    fn read_vec3(&mut self) -> Result<Vec3> {
        let x = self.input.read_f32::<LittleEndian>()?;
        let y = self.input.read_f32::<LittleEndian>()?;
        let z = self.input.read_f32::<LittleEndian>()?;

        Ok(Vec3::new(f64::from(x), f64::from(y), f64::from(z)))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        guess_stl_format, load_ascii_stl, load_binary_stl, Result, StlFormat, StlTriangle, Vec3,
    };

    use std::io::{BufReader, Cursor};

    #[test]
    fn test_load_binary_stl() {
        let cube_stl = include_bytes!("../../data/cube.stl");
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

        let tris = tris.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(
            tris,
            vec![
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                    positions: [
                        Vec3::new(-1.0, -1.0, -1.0),
                        Vec3::new(-1.0, -1.0, 1.0),
                        Vec3::new(-1.0, 1.0, 1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                    positions: [
                        Vec3::new(-1.0, 1.0, 1.0),
                        Vec3::new(-1.0, 1.0, -1.0),
                        Vec3::new(-1.0, -1.0, -1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    positions: [
                        Vec3::new(-1.0, 1.0, -1.0),
                        Vec3::new(-1.0, 1.0, 1.0),
                        Vec3::new(1.0, 1.0, 1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 1.0, 0.0),
                    positions: [
                        Vec3::new(1.0, 1.0, 1.0),
                        Vec3::new(1.0, 1.0, -1.0),
                        Vec3::new(-1.0, 1.0, -1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(1.0, 0.0, 0.0),
                    positions: [
                        Vec3::new(1.0, 1.0, -1.0),
                        Vec3::new(1.0, 1.0, 1.0),
                        Vec3::new(1.0, -1.0, 1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(1.0, 0.0, 0.0),
                    positions: [
                        Vec3::new(1.0, -1.0, 1.0),
                        Vec3::new(1.0, -1.0, -1.0),
                        Vec3::new(1.0, 1.0, -1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, -1.0, 0.0),
                    positions: [
                        Vec3::new(-1.0, -1.0, 1.0),
                        Vec3::new(-1.0, -1.0, -1.0),
                        Vec3::new(1.0, -1.0, -1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, -1.0, 0.0),
                    positions: [
                        Vec3::new(1.0, -1.0, -1.0),
                        Vec3::new(1.0, -1.0, 1.0),
                        Vec3::new(-1.0, -1.0, 1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    positions: [
                        Vec3::new(1.0, -1.0, -1.0),
                        Vec3::new(-1.0, -1.0, -1.0),
                        Vec3::new(-1.0, 1.0, -1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    positions: [
                        Vec3::new(-1.0, 1.0, -1.0),
                        Vec3::new(1.0, 1.0, -1.0),
                        Vec3::new(1.0, -1.0, -1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    positions: [
                        Vec3::new(1.0, 1.0, 1.0),
                        Vec3::new(-1.0, 1.0, 1.0),
                        Vec3::new(-1.0, -1.0, 1.0),
                    ],
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 0.0, 1.0),
                    positions: [
                        Vec3::new(-1.0, -1.0, 1.0),
                        Vec3::new(1.0, -1.0, 1.0),
                        Vec3::new(1.0, 1.0, 1.0),
                    ],
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
            guess_stl_format(&mut BufReader::new(Cursor::new(&stl[..]))).unwrap(),
            StlFormat::Ascii
        );

        let (name, tris) = load_ascii_stl(stl).unwrap();

        assert_eq!(name, "cube_corner");
        assert_eq!(
            tris,
            vec![
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, -1.0, 0.0),
                    positions: [
                        Vec3::zero(),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0)
                    ]
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.0, 0.0, -1.0),
                    positions: [
                        Vec3::zero(),
                        Vec3::new(0.0, 1.0, 0.0),
                        Vec3::new(1.0, 0.0, 0.0)
                    ]
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(-1.0, 0.0, 0.0),
                    positions: [
                        Vec3::zero(),
                        Vec3::new(0.0, 0.0, 1.0),
                        Vec3::new(0.0, 1.0, 0.0)
                    ]
                },
                StlTriangle {
                    attributes: vec![],
                    normal: Vec3::new(0.577, 0.577, 0.577),
                    positions: [
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.0, 1.0, 0.0),
                        Vec3::new(0.0, 0.0, 1.0)
                    ]
                }
            ]
        )
    }
}
