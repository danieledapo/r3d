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
