use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::with_dimensions_hint(500, 500, 500);

    for y in (-500..=500).step_by(50) {
        let yt = f64::from(y) / 500.0;
        for x in (-500..=500).step_by(50) {
            let xt = f64::from(x) / 500.0;
            let t = 1.0 - f64::abs(xt * yt);
            let h = 150.0 + 300.0 * t.powi(6);

            scene.zslab((x + 25, y + 25, 0), (20, 20, h as i32));
            scene.zslab((x + 25, y + 25, -h as i32), (20, 20, h as i32));
        }
    }

    let triangles = render_outlines(&scene);

    dump_outlines_svg("sym.svg", &triangles, &SvgSettings::new(1080.0, 1920.0))
        .expect("cannot save sym.svg");

    opener::open("sym.svg").expect("cannot open sym.svg");
}
