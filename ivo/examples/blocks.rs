use sketch_utils::opener;

use rand::prelude::*;

use ivo::*;

type Blob = (i32, i32, i32, i32, i32, i32);

pub fn cube(rng: &mut impl Rng, n: usize, perfect_grid: bool) -> Vec<Blob> {
    let mut blobs = vec![];

    for _ in 0..n {
        let x = rng.gen_range(6..=14);
        let y = rng.gen_range(6..=14);
        let z = rng.gen_range(6..=14);

        let mut dim = |c, s, e| {
            let d;
            if perfect_grid {
                d = i32::min(c - s, e - c);
            } else {
                d = i32::max(c - s, e - c);
            }
            rng.gen_range(d / 2..=d)
        };

        let w = dim(x, 4, 16);
        let h = dim(y, 4, 16);
        let d = dim(z, 4, 16);

        blobs.push((x, y, z, w, h, d));
    }

    blobs
}

pub fn main() {
    let mut scene = Scene::new();

    let mut args = std::env::args().skip(1);
    let mut next_arg = |def| args.next().map(|l| l.parse().unwrap()).unwrap_or(def);
    let n = next_arg(10);
    let perfect_grid = next_arg(0) != 0;
    let invert = next_arg(0) != 0;
    let size = next_arg(10) as i32;

    let mut rng = rand::thread_rng();
    for i in 0..size {
        let bx = i * 20;

        let mut j = 0;
        while j < size {
            let blobs = cube(&mut rng, n, perfect_grid);

            let gg = rng.gen_range(2..=6);
            for k in 0..gg {
                if j + k >= size {
                    break;
                }

                for &(x, y, z, w, h, d) in &blobs {
                    scene.aabb((bx + x, y + (j + k) * 20, z), (w, h, d));

                    if invert {
                        scene.invert();
                    }
                }
            }

            j += gg;
        }
    }

    let triangles = render(&scene);

    dump_svg("blocks.svg", &triangles, &SvgSettings::new(2048.0, 2048.0))
        .expect("cannot save blocks.svg");

    opener::open("blocks.svg").expect("cannot open blocks.svg");
}
