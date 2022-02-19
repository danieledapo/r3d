use geo::util::opener;
use isovoxel::*;

pub fn main() {
    let mut voxels = vec![];

    let n = 33_i32;
    for z in -n..=n {
        for y in -n..=n {
            for x in -n..=n {
                let pd = f64::sqrt(f64::from(x.pow(2) + y.pow(2) + z.pow(2)));
                let d = f64::max(pd - f64::from(n - 3), -f64::from(z) - 5.0);
                if d <= 0.0 {
                    voxels.push(Voxel::new(x, y, z));
                }
            }
        }
    }
    for y in -n..n {
        for x in -n..n {
            voxels.push(Voxel::new(x, y, -5));
        }
    }

    for z in 0..=40 {
        voxels.push(Voxel::new(0, 0, z));
    }
    for z in -3_i32..=3 {
        for y in -3_i32..=3 {
            for x in -3_i32..=3 {
                let pd = f64::sqrt(f64::from(x.pow(2) + y.pow(2) + z.pow(2)));
                if pd - 2.0 <= 0.0 {
                    voxels.push(Voxel::new(x, y, 40 + z));
                }
            }
        }
    }

    let triangles = render(voxels);

    dump_svg(
        "sphere.svg",
        &triangles,
        &SvgSettings {
            background: Some("white"),
            scale: 10.0,
            stroke: "black",
            stroke_width: 0.1,
            digits: 4,
            padding: 2.0,
        },
    )
    .expect("cannot save sphere.svg");

    opener::open("sphere.svg").expect("cannot open sphere.svg");
}
