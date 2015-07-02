#![crate_type = "lib"]

#![crate_name = "histogram"]

/* 
	This crate is inspired by HdrHistogram.

	WORK IN PROGRESS
*/

#![feature(vec_resize)]
#![feature(test)]

extern crate test;

// TODO(brayniac): u64 is likely overkill
// TODO(brayniac): add other counters

#[allow(dead_code)]
pub struct Histogram {
    total: u64,
    precision: u32,
    inner_buckets: u64,
    max_value: u64,
    data: Vec<u64>,
}

impl Histogram {
    /// return a new Histogram
    pub fn new(precision: u32) -> Option<Histogram> {
        let max_value = 3600000000;
        let inner_buckets = 10_u64.pow(precision);
        let outer_buckets = max_value as f64;
        let outer_buckets = outer_buckets.log2().ceil() as u64;

        let total_buckets = (inner_buckets * outer_buckets) as usize;

        let mut data = Vec::with_capacity(total_buckets);
        data.resize(total_buckets, 0);

        Some(Histogram {
            total: 0,
            data: data,
            precision: precision,
            inner_buckets: inner_buckets,
            max_value: max_value,
        })
    }

    /// increment counters for value's bucket
    pub fn increment(&mut self, value: u64) {
        self.total += 1;
        let index = self.get_index(value);
        self.data[index] += 1;
    }

    /// get the count at some value
    pub fn get(&mut self, value: u64) -> u64 {
        let index = self.get_index(value);
        self.data[index]
    }

    pub fn get_index(&mut self, value: u64) -> usize {

        let mut index = 0;

        if value >= 1 {
            let outer = 63 - value.leading_zeros();
            let inner = (value as f64 / 2.0_f64.powi(outer as i32) - 1.0_f64) *
                        (self.inner_buckets as f64);

            let outer = outer as u64;
            let inner = inner.ceil() as u64;
            index = (outer * self.inner_buckets) + inner + 1;
        }

        index as usize
    }
}

#[cfg(test)]
mod tests {
    use super::{Histogram};
    use test::Bencher;

    #[test]
    fn it_works() {
    }

    #[test]
    fn test_new() {
        let histogram = Histogram::new(6).unwrap();

        assert!(histogram.total == 0);
    }

    #[test]
    fn test_increment() {
        let mut histogram = Histogram::new(6).unwrap();

        histogram.increment(0);
        assert!(histogram.total == 1);
        histogram.increment(0);
        assert!(histogram.total == 2);
    }

    #[test]
    fn test_get() {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(1);
        assert!(histogram.get(1) == 1);

        histogram.increment(1);
        assert!(histogram.get(1) == 2);

        histogram.increment(2);
        assert!(histogram.get(2) == 1);

        assert!(histogram.get(3) == 0);
    }

    #[test]
    fn test_get_index() {
        let mut histogram = Histogram::new(3).unwrap();

        assert_eq!(histogram.get_index(1), 1);
        assert_eq!(histogram.get_index(2), 1001);
        assert_eq!(histogram.get_index(3), 1501);
        assert_eq!(histogram.get_index(1023), 10000);
        assert_eq!(histogram.get_index(1024), 10001);
        assert_eq!(histogram.get_index(1025), 10002);
    }

    #[bench]
    fn bench_get_small_value(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(1);

        b.iter(|| {
            histogram.get(1)
        })
    }

    #[bench]
    fn bench_get_large_value(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(3600000000);

        b.iter(|| {
            histogram.get(3600000000)
        })
    }

    #[bench]
    fn bench_increment_small_value(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(1);

        b.iter(|| {
            histogram.increment(1)
        })
    }

    #[bench]
    fn bench_increment_large_value(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(3600000000);

        b.iter(|| {
            histogram.increment(3600000000)
        })
    }

    #[bench]
    fn bench_get_index_small_value(b: &mut Bencher) {
        let mut histogram = Histogram::new(6).unwrap();

        b.iter(|| {
            histogram.get_index(1);
        })
    }

    #[bench]
    fn bench_get_index_large_value(b: &mut Bencher) {
        let mut histogram = Histogram::new(6).unwrap();

        b.iter(|| {
            histogram.get_index(3600000000);
        })
    }
}
