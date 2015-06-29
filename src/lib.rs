#![crate_type = "lib"]

#![crate_name = "histogram"]

/* 
	This crate is inspired by HdrHistogram.

	WORK IN PROGRESS
*/

use std::collections::HashMap;

pub struct Histogram {
    total: u64,
    data: HashMap<u64, u64>
}

impl Histogram {
    /// return nil or empty Histogram
    pub fn nil() -> Histogram {
        Histogram {
            total: 0,
            data: HashMap::new(),
        }
    }

    /// return a new Histogram
    pub fn new() -> Option<Histogram> {
        let histogram = Histogram::nil();
        Some(histogram)
    }

    pub fn store(&mut self, value: u64) {
        self.total += 1;
        let count = self.data.entry(value).or_insert(0);
        *count += 1;
    }

    pub fn bucket_count(&mut self, value: u64) -> u64 {
        let key: u64 = value;
        let count = self.data.get(&key);
        match count {
            Some(v) => *v,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Histogram};

    #[test]
    fn it_works() {
    }

    #[test]
    fn test_nil() {
        let nil = Histogram::nil();

        assert!(nil.total == 0);
    }

    #[test]
    fn test_new() {
        let histogram = Histogram::new().unwrap();

        assert!(histogram.total == 0);
    }

    #[test]
    fn test_store() {
        let mut histogram = Histogram::new().unwrap();

        histogram.store(0);
        assert!(histogram.total == 1);
        histogram.store(0);
        assert!(histogram.total == 2);
    }

    #[test]
    fn test_get_count() {
        let mut histogram = Histogram::new().unwrap();

        histogram.store(0);
        assert!(histogram.bucket_count(0) == 1);

        histogram.store(0);
        assert!(histogram.bucket_count(0) == 2);

        histogram.store(1);
        assert!(histogram.bucket_count(1) == 1);

        assert!(histogram.bucket_count(2) == 0);
    }
}
