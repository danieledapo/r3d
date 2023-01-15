use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

pub struct Image<const PIXELS: usize> {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image<3> {
    pub fn rgb(w: u32, h: u32) -> Self {
        let d = usize::try_from(w).unwrap() * usize::try_from(h).unwrap();
        Self {
            data: vec![0; d * 3],
            width: w,
            height: h,
        }
    }
}

impl Image<1> {
    pub fn grayscale(w: u32, h: u32) -> Self {
        let d = usize::try_from(w).unwrap() * usize::try_from(h).unwrap();
        Self {
            data: vec![0; d],
            width: w,
            height: h,
        }
    }
}

impl<const PIXELS: usize> Image<PIXELS> {
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    pub fn pixels_mut(&mut self) -> impl Iterator<Item = (u32, u32, &mut [u8])> {
        let w = usize::try_from(self.width).unwrap();
        self.data
            .chunks_exact_mut(PIXELS)
            .enumerate()
            .map(move |(i, p)| ((i % w) as u32, (i / w) as u32, p))
    }

    pub fn save(&self, f: &str) -> io::Result<()> {
        let header = if PIXELS == 1 {
            assert!(f.ends_with(".pgm"));
            "P5"
        } else if PIXELS == 3 {
            assert!(f.ends_with(".ppm"));
            "P6"
        } else {
            unimplemented!();
        };

        let mut out = BufWriter::new(File::create(f)?);
        writeln!(out, "{header}")?;

        writeln!(out, "{} {}", self.width, self.height)?;
        writeln!(out, "255")?;

        out.write_all(&self.data)?;

        Ok(())
    }
}
