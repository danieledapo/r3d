use std::{convert::TryFrom, io::BufRead};

use crate::{v3, Triangle, Vec3};

use super::{Error, Mesh, Result};

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

impl Obj {
    /// Try to load an `Obj` from the given reader.
    pub fn load(r: impl BufRead) -> Result<Obj> {
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
                    let x: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;
                    let y: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;
                    let z: f64 = tokens.next().ok_or(Error::BadFormat)?.parse()?;

                    mesh.vertices.push(v3(x, y, z));
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
                    if l.len() != 3 {
                        return Err(Error::BadFormat);
                    }

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
}

impl Mesh for Obj {
    fn triangles(&self) -> Box<dyn Iterator<Item = Triangle> + '_> {
        Box::new(self.loops.iter().map(move |l| {
            let get_v = |i: isize| {
                self.vertices[if i > 0 {
                    usize::try_from(i - 1).unwrap()
                } else {
                    self.vertices.len() - usize::try_from(i.abs()).unwrap()
                }]
            };

            Triangle::new(get_v(l[0]), get_v(l[1]), get_v(l[2]))
        }))
    }
}
