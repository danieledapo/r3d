use geo::util::opener;
use isovoxel::*;

pub fn main() {
    let mut voxels = vec![];

    for i in 0..10 {
        voxels.push(Voxel::new(i, 0, 9));
        voxels.push(Voxel::new(0, i, 9));
        for j in 0..10 {
            voxels.push(Voxel::new(i, j, 0));
        }
    }

    for j in 0..10 {
        voxels.push(Voxel::new(0, 0, j));
        voxels.push(Voxel::new(9, 0, j));
        voxels.push(Voxel::new(0, 9, j));
    }

    for i in 0..=5 {
        voxels.push(Voxel::new(9, i, 9));
        voxels.push(Voxel::new(i, 9, 9));
        voxels.push(Voxel::new(9, 9, i));
    }

    let triangles = render(voxels);

    dump_svg(
        "hollow_cube.svg",
        &triangles,
        &SvgSettings {
            background: Some("white"),
            scale: 50.0,
            stroke: "black",
            stroke_width: 0.1,
            digits: 4,
            padding: 2.0,
        },
    )
    .expect("cannot save hollow_cube.svg");

    opener::open("hollow_cube.svg").expect("cannot open hollow_cube.svg");
}
