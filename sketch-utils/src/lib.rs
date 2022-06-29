use std::{
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub mod opener;

#[macro_export]
macro_rules! chrono {
    ($label:expr, $e:expr) => {{
        let now = std::time::Instant::now();
        let res = $e;
        println!("{} took {}ms", $label, now.elapsed().as_millis());
        res
    }};
}

pub fn sketch_output_path(sketch: &str) -> io::Result<String> {
    let sketch = Path::new(sketch);
    let name = sketch.file_stem().unwrap();
    let ext = sketch.extension().unwrap();

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name);

    if !dir.exists() {
        fs::create_dir(&dir)?;
    }

    let name = format!(
        "{}-{}.{}",
        name.to_str().unwrap(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        ext.to_str().unwrap()
    );

    Ok(dir.join(name).to_str().unwrap().to_owned())
}
