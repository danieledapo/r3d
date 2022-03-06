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

pub fn arange(s: f64, e: f64, step: f64) -> impl Iterator<Item = f64> {
    Arange { s, e, step }
}

#[derive(Debug, Clone)]
struct Arange {
    s: f64,
    e: f64,
    step: f64,
}

impl Iterator for Arange {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s > self.e {
            return None;
        }

        let v = self.s;
        self.s += self.step;
        Some(v)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = f64::ceil((self.e - self.s) / self.step) as u64;
        (0, usize::try_from(n).ok())
    }
}
