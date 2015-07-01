#![crate_type = "lib"]

#![crate_name = "histogram"]

/* 
	This crate is inspired by HdrHistogram.

	WORK IN PROGRESS
*/

#![feature(test)]
extern crate test;


use std::collections::HashMap;

// TODO(brayniac): u64 is likely overkill
// TODO(brayniac): add other counters
pub struct Histogram {
    total: u64,
    precision: u32,
    inner_buckets: u64,
    max_value: u64,
    data: [u64],
}

struct Bucket {
    outer: u64,
    inner: u64,
}

impl Histogram {
    /// return a new Histogram
    pub fn new(precision: u32, max_value: u64) -> Option<Histogram> {
        let inner_buckets = 10_u64.pow(precision)
        let outer_buckets = max_value as f64;
        outerbuckets = outer_buckets.log2().ceil() as u64;

        let total_buckets = inner_buckets * outer_buckets;

        Some(Histogram {
            total: 0,
            data: HashMap::new(),
            precision: precision,
            inner_buckets: 10_u64.pow(precision)
        })
    }

    /// increment counters for value's bucket
    pub fn increment(&mut self, value: u64) {
        self.total += 1;
        match self.get_bucket(value) {
            Ok(bucket) => {
                let count = self.data.entry(bucket.outer).or_insert(HashMap::new()).entry(bucket.inner).or_insert(0);
                *count += 1;
            },
            Err(_) => {

            }
        }
    }

    /// get the count at some value
    pub fn get(&mut self, value: u64) -> u64 {
        match self.get_bucket(value) {
            Ok(bucket) => {
                match self.data.get(&bucket.outer) {
                    Some(outer) => {
                        match outer.get(&bucket.inner) {
                            Some(count) => *count,
                            None => 0,
                        }
                    },
                    None => 0,
                }
            },
            Err(_) => 0
        }
    }

    fn get_bucket_old(&mut self, value: u64) -> Result<Bucket, &'static str> {
        if value == 0 {
            return Err("value too small");
        }

        let v = value as f64;

        // outer bucket is log2(n)
        let outer = v.log2().floor();

 
         // inner is linearly scaled between 2^(outer) and 2**(outer+1)
        let inner = (v / 2.0_f64.powf(outer) - 1.0_f64) * (self.inner_buckets as f64);
        Ok(Bucket {
            outer: outer.floor() as u64,
            inner: inner.ceil() as u64,
        })

    }




    fn get_bucket(&mut self, value: u64) -> Result<Bucket, &'static str> {

        if value == 0 {
            return Err("value too small");
        }

        let v = value as f64;

        let outer = 63 - value.leading_zeros();

        let remaining = (value - 2_u64.pow(outer));

        // inner is linearly scaled between 2^(outer) and 2**(outer+1)
        let inner = (remaining as f64 / 2_u64.pow(outer) as f64) * (self.inner_buckets as f64);

        Ok(Bucket {
            outer: outer as u64,
            inner: inner.ceil() as u64,
        })
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
        let histogram = Histogram::new(1).unwrap();

        assert!(histogram.total == 0);
    }

    #[test]
    fn test_increment() {
        let mut histogram = Histogram::new(1).unwrap();

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
    fn test_get_bucket() {
        let mut histogram = Histogram::new(1).unwrap();

        match histogram.get_bucket(0) {
            Ok(_) => assert!(false, "value 0 shouldn't have a Bucket"),
            Err(e) => assert_eq!(e, "value too small"),
        }

        let bucket = histogram.get_bucket(1).unwrap();
        assert_eq!(bucket.outer, 0);
        assert_eq!(bucket.inner, 0);

        let bucket = histogram.get_bucket(2).unwrap();
        assert_eq!(bucket.outer, 1);
        assert_eq!(bucket.inner, 0);

        let bucket = histogram.get_bucket(3).unwrap();
        assert_eq!(bucket.outer, 1);
        assert_eq!(bucket.inner, 5);

        let bucket = histogram.get_bucket(1023).unwrap();
        assert_eq!(bucket.outer, 9);
        assert_eq!(bucket.inner, 10);

        let bucket = histogram.get_bucket(1024).unwrap();
        assert_eq!(bucket.outer, 10);
        assert_eq!(bucket.inner, 0);

        let bucket = histogram.get_bucket(1025).unwrap();
        assert_eq!(bucket.outer, 10);
        assert_eq!(bucket.inner, 1);
    }

    

    #[bench]
    fn bench_get_bucket(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(1);

        b.iter(|| {
            histogram.get_bucket(1)
        })

    }

    #[bench]
    fn bench_get_bucket_max(b: &mut Bencher) {
        let mut histogram = Histogram::new(10).unwrap();

        histogram.increment(3600000000);

        b.iter(|| {
            histogram.get_bucket(3600000000)
        })

    }

    #[bench]
    fn bench_get_bucket_old(b: &mut Bencher) {
        let mut histogram = Histogram::new(10).unwrap();

        histogram.increment(1);

        b.iter(|| {
            histogram.get_bucket_old(1)
        })

    }

    #[bench]
    fn bench_get_bucket_old_max(b: &mut Bencher) {
        let mut histogram = Histogram::new(10).unwrap();

        histogram.increment(3600000000);

        b.iter(|| {
            histogram.get_bucket_old(3600000000)
        })

    }

    #[bench]
    fn bench_get(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(1);

        b.iter(|| {
            histogram.get(1)
        })

    }

    #[bench]
    fn bench_increment(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(1);

        b.iter(|| {
            histogram.increment(1)
        })

    }

    #[bench]
    fn bench_incremen_max(b: &mut Bencher) {
        let mut histogram = Histogram::new(1).unwrap();

        histogram.increment(3600000000);

        b.iter(|| {
            histogram.increment(3600000000)
        })

    }
}
