use std::io;

use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    for l in io::stdin().lines() {
        let l = l.unwrap();

        let mut coords = l.split(',');
        let x = coords.next().unwrap().parse::<i32>().unwrap();
        let y = coords.next().unwrap().parse::<i32>().unwrap();
        let z = coords.next().unwrap().parse::<i32>().unwrap();

        scene.add(x, y, z);
    }

    let triangles = render_outlines(&scene);

    dump_outlines_svg("voxels.svg", &triangles, &SvgSettings::new(2048.0, 2048.0))
        .expect("cannot save voxels.svg");

    opener::open("voxels.svg").expect("cannot open voxels.svg");
}
