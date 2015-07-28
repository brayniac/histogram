//! A native rust implementation of a histogram and percentiles which can offer
//! specified precision across the full range of stored values. This library is
//! inspired by the HdrHistogram project.
//!
//!
//! # Goals
//! * maintain precision across full value range
//! * provide percentile metrics for stored data
//! * pre-allocated datastructure
//!
//! # Future work
//! * unknown
//!
//! # Usage
//!
//! Create a new histogram, call increment for every value, retrieve percentile
//! stats.
//!
//! # Example
//!
//! Create a histogram, increment values, show percentiles
//!
//! ```
//!
//! use histogram::*;
//!
//! let mut histogram = Histogram::new(
//!     HistogramConfig {
//!         precision: 4,       // maintain > 4 sigfigs (max error .01%)
//!         max_value: 1000000, // max storable value. fewer, less ram needed
//!         max_memory: 0,      // optional memory bound in Bytes. 0 = unlimited
//!     }
//! ).unwrap();
//!
//! let mut value = 0;
//!
//! for i in 1..100 {
//!     histogram.increment(i);
//! }
//!
//! // print percentiles from the histogram
//! println!("Percentiles: p50: {} ns p90: {} ns p99: {} ns p999: {}",
//!     histogram.percentile(50.0).unwrap(),
//!     histogram.percentile(90.0).unwrap(),
//!     histogram.percentile(99.0).unwrap(),
//!     histogram.percentile(99.9).unwrap(),
//! );

#![crate_type = "lib"]

#![crate_name = "histogram"]

#[derive(Default)]
pub struct HistogramConfig {
    pub precision: u32,
    pub max_memory: u32,
    pub max_value: u64,
}

#[derive(Default)]
pub struct HistogramCounters {
    entries_total: u64,
    missed_unknown: u64,
    missed_small: u64,
    missed_large: u64,
}

pub struct HistogramData {
    data: Vec<u64>,
    counters: HistogramCounters,
    iterator: usize,
}

#[allow(dead_code)]
pub struct HistogramProperties {
    buckets_inner: u32,
    buckets_outer: u32,
    buckets_total: u32,
    memory_used: u32,
    linear_max: u64,
    linear_power: u32,
}

pub struct Histogram {
    config: HistogramConfig,
    data: HistogramData,
    properties: HistogramProperties,
}

pub struct HistogramBucket {
    pub value: u64,
    pub count: u64,
    pub id: u64,
}

impl Iterator for Histogram {
    type Item = HistogramBucket;

    fn next(&mut self) -> Option<HistogramBucket> {
        let current = self.data.iterator;
        self.data.iterator += 1;

        if current == (self.properties.buckets_total as usize) {
            self.data.iterator = 0;
            None
        } else {
            //println!("Iterating: current: {}", current);
            Some(HistogramBucket {
                id: current as u64,
                value: self.index_value(current),
                count: self.data.data[current],
            })
        }
    }
}

impl Histogram {

    /// create a new Histogram
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    ///
    /// let mut h = Histogram::new(
    ///     HistogramConfig{
    ///         max_value: 1000000,
    ///         precision: 3,
    ///         max_memory: 0,
    /// }).unwrap();
    pub fn new(config: HistogramConfig) -> Option<Histogram> {

        let radix = 10_u32;

        let buckets_inner: u32 = radix.pow(config.precision);

        let linear_power: u32 = 32 - buckets_inner.leading_zeros();

        let linear_max: u64 = 2.0_f64.powi(linear_power as i32) as u64 - 1;

        let max_value_power: u32 = 64 - config.max_value.leading_zeros();

        let mut buckets_outer = 0;

        if max_value_power > linear_power {
            buckets_outer = max_value_power - linear_power;
        }

        let buckets_total = buckets_inner * buckets_outer + linear_max as u32;

        let memory_used = buckets_total * 8;

        if config.max_memory > 0 && config.max_memory < memory_used {
            return None;
        }

        let mut data = Vec::with_capacity(buckets_total as usize);

        // vector is already sized to fit, just set the length accordingly
        unsafe {
            data.set_len(buckets_total as usize);
        }

        let counters: HistogramCounters = Default::default();

        Some(Histogram {
            config: config,
            data: HistogramData {
                data: data,
                counters: counters,
                iterator: 0,
            },
            properties: HistogramProperties {
                buckets_inner: buckets_inner,
                buckets_outer: buckets_outer,
                buckets_total: buckets_total,
                memory_used: memory_used,
                linear_max: linear_max,
                linear_power: linear_power,
            },
        })
    }

    /// increment the count for a value
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    ///
    /// let mut h = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// h.increment(1);
    /// assert_eq!(h.get(1).unwrap(), 1);
    pub fn increment(&mut self, value: u64) {
        self.data.counters.entries_total = self.data.counters.entries_total.saturating_add(1_u64);
        if value < 1 {
            self.data.counters.missed_small =
                self.data.counters.missed_small.saturating_add(1_u64);
        } else if value > self.config.max_value {
            self.data.counters.missed_large =
                self.data.counters.missed_large.saturating_add(1_u64);
        } else {
            match self.get_index(value) {
                Some(index) => {
                    self.data.data[index] = self.data.data[index].saturating_add(1_u64);
                },
                None => {
                    self.data.counters.missed_unknown =
                        self.data.counters.missed_unknown.saturating_add(1_u64);
                }
            }
        }
    }

    /// record additional counts for value
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    ///
    /// let mut h = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// h.record(1, 1);
    /// assert_eq!(h.get(1).unwrap(), 1);
    ///
    /// h.record(2, 2);
    /// assert_eq!(h.get(2).unwrap(), 2);
    ///
    /// h.record(10, 10);
    /// assert_eq!(h.get(10).unwrap(), 10);
    pub fn record(&mut self, value: u64, count: u64) {
        self.data.counters.entries_total = self.data.counters.entries_total.saturating_add(count);
        if value < 1 {
            self.data.counters.missed_small =
                self.data.counters.missed_small.saturating_add(count);
        } else if value > self.config.max_value {
            self.data.counters.missed_large =
                self.data.counters.missed_large.saturating_add(count);
        } else {
            match self.get_index(value) {
                Some(index) => {
                    self.data.data[index] = self.data.data[index].saturating_add(count);
                },
                None => {
                    self.data.counters.missed_unknown =
                        self.data.counters.missed_unknown.saturating_add(count);
                }
            }
        }
    }

    /// get the count for a value
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    ///
    /// let mut h = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// assert_eq!(h.get(1).unwrap(), 0);
    pub fn get(&mut self, value: u64) -> Option<u64> {
        match self.get_index(value) {
            Some(index) => {
                return Some(self.data.data[index]);
            },
            None => {
                None
            }
        }
    }

    // calculate the index for a given value
    fn get_index(&mut self, value: u64) -> Option<usize> {
        let result: Option<usize> = None;

        if value >= 1 {

            if value <= self.properties.linear_max {
                return Some((value - 1) as usize);
            }

            let l_max = self.properties.linear_max as u32;

            let outer = 63 - value.leading_zeros();

            let l_power = 64 - self.properties.linear_max.leading_zeros();

            let remain = value as f64 - 2.0_f64.powi(outer as i32);

            let inner = (self.properties.buckets_inner as f64 * remain as f64 /
                         2.0_f64.powi((outer) as i32)).floor() as u32;

            println!("Value: {} Outer: {} l_max: {} l_power: {} Remain: {} Inner: {}", value,
                     outer, l_max, l_power, remain, inner);

            // this gives the shifted outer index
            let outer = outer as u32 - l_power;

            let index = l_max + self.properties.buckets_inner * outer + inner;

            return Some(index as usize);
        }
        result
    }

    // calculate the nominal value of the given index
    fn index_value(&mut self, index: usize) -> u64 {

        // in this case, the index is linear
        let index = index as u32;

        let linear_max = self.properties.linear_max as u32;

        if index < linear_max {
            return (index + 1) as u64;
        }

        let log_index = index - linear_max;

        let outer = (log_index as f64 / self.properties.buckets_inner as f64).floor() as u32;

        let inner = log_index - outer * self.properties.buckets_inner as u32;

        let mut value = 2.0_f64.powi((outer as u32 + self.properties.linear_power) as i32);
        value += inner as f64 * (value as f64 / self.properties.buckets_inner as f64);

        value.ceil() as u64
    }

    /// return the value for the given percentile
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    /// let mut h = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// for value in 1..1000 {
    ///     h.increment(value);
    /// }
    ///
    /// assert_eq!(h.percentile(50.0).unwrap(), 501);
    /// assert_eq!(h.percentile(90.0).unwrap(), 901);
    /// assert_eq!(h.percentile(99.0).unwrap(), 991);
    /// assert_eq!(h.percentile(99.9).unwrap(), 999);
    pub fn percentile(&mut self, percentile: f64) -> Option<u64> {

        if self.entries() < 1 {
            return None;
        }

        if percentile <= 100.0 && percentile >= 0.0 {

            let total = self.data.counters.entries_total;

            let mut need = (total as f64 * (percentile / 100.0_f64)).ceil() as u64;

            if need > total {
                need = total;
            }

            need = total - need;

            if need == 0 {
                need = 1;
            }

            let mut index: isize = (self.properties.buckets_total - 1) as isize;
            let mut step: isize = -1 as isize;
            let mut have: u64 = 0 as u64;

            if percentile < 50.0 {
                index = 0 as isize;
                step = 1 as isize;
                need = total - need;
            }

            loop {
                have = have + self.data.data[index as usize];

                if have >= need {
                    return Some(self.index_value(index as usize) as u64);
                }

                index += step;

                if index > self.properties.buckets_total as isize {
                    break;
                }
                if index < 0 {
                    break;
                }
            }
        }
        None
    }

    /// merge one Histogram into another Histogram
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    ///
    /// let mut a = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// let mut b = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// assert_eq!(a.entries(), 0);
    /// assert_eq!(b.entries(), 0);
    ///
    /// a.increment(1);
    /// b.increment(2);
    ///
    /// assert_eq!(a.entries(), 1);
    /// assert_eq!(b.entries(), 1);
    ///
    /// a.merge(&mut b);
    ///
    /// assert_eq!(a.entries(), 2);
    /// assert_eq!(a.get(1).unwrap(), 1);
    /// assert_eq!(a.get(2).unwrap(), 1);
    pub fn merge(&mut self, other: &mut Histogram) {
        loop {
            match other.next() {
                Some(bucket) => {
                    self.record(bucket.value, bucket.count);
                }
                None => { break }
            }
        }
    }

    /// return the number of entries in the Histogram
    ///
    /// # Example
    /// ```
    /// # use histogram::{Histogram,HistogramConfig};
    ///
    /// let mut h = Histogram::new(
    ///     HistogramConfig{
    ///         max_memory: 0,
    ///         max_value: 1000000,
    ///         precision: 3,
    /// }).unwrap();
    ///
    /// assert_eq!(h.entries(), 0);
    /// h.increment(1);
    /// assert_eq!(h.entries(), 1);
    pub fn entries(&mut self) -> u64 {
        self.data.counters.entries_total
    }
}

#[cfg(test)]
mod tests {
    use super::{Histogram, HistogramConfig};

    #[test]
    fn test_new_0() {
        // this histogram has only a linear region which runs 1-15

        let h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 10,
            precision: 1,
        }).unwrap();

        assert_eq!(h.properties.buckets_inner, 10); // 10 ^ precision
        assert_eq!(h.properties.buckets_outer, 0); // max <= 2 * buckets_inner
        assert_eq!(h.properties.buckets_total, 15); // only linear region
    }

    #[test]
    fn test_new_1() {
        // this histogram has linear and log regios

        let h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 31,
            precision: 1,
        }).unwrap();

        assert_eq!(h.properties.buckets_inner, 10); // 10 ^ precision
        assert_eq!(h.properties.buckets_outer, 1); // max <= 2 * buckets_inner
        assert_eq!(h.properties.buckets_total, 25); // only linear region
    }

    #[test]
    fn test_new_2() {
        let h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 32,
            precision: 1,
        }).unwrap();

        assert_eq!(h.properties.buckets_inner, 10); // 10 ^ precision
        assert_eq!(h.properties.buckets_outer, 2); // max <= 2 * buckets_inner
        assert_eq!(h.properties.buckets_total, 35); // only linear region
    }

    #[test]
    fn test_new_3() {
        let h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 10000,
            precision: 3,
        }).unwrap();

        assert_eq!(h.properties.buckets_inner, 1000); // 10 ^ precision
        assert_eq!(h.properties.buckets_outer, 4); // max <= 2 * buckets_inner
        assert_eq!(h.properties.buckets_total, 5023); // only linear region
    }

    #[test]
    fn test_increment_0() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 10,
            precision: 3,
        }).unwrap();

        for op in 1..1000000 {
            h.increment(1);
            assert_eq!(h.entries(), op);
        }
    }

    #[test]
    fn test_increment_1() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 10,
            precision: 3,
        }).unwrap();

        // increment values across the entire range
        // including 0 and > max_value
        for v in 0..11 {
            h.increment(v);
            assert_eq!(h.entries(), (v + 1));
        }
    }

    #[test]
    fn test_get() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 10,
            precision: 1,
        }).unwrap();

        h.increment(1);
        assert_eq!(h.get(1), Some(1));

        h.increment(1);
        assert_eq!(h.get(1), Some(2));

        h.increment(2);
        assert_eq!(h.get(2), Some(1));

        assert_eq!(h.get(3), Some(0));
    }

    #[test]
    fn test_get_index_0() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 32,
            precision: 3,
        }).unwrap();

        // all values should index directly to (value - 1)
        // no estimated buckets are needed given the precision and max

        assert_eq!(h.get_index(1), Some(0));
        assert_eq!(h.index_value(0), 1);

        assert_eq!(h.get_index(2), Some(1));
        assert_eq!(h.index_value(1), 2);

        assert_eq!(h.get_index(3), Some(2));
        assert_eq!(h.index_value(2), 3);

        assert_eq!(h.get_index(4), Some(3));
        assert_eq!(h.index_value(3), 4);

        assert_eq!(h.get_index(5), Some(4));
        assert_eq!(h.index_value(4), 5);

        assert_eq!(h.get_index(16), Some(15));
        assert_eq!(h.index_value(15), 16);

        assert_eq!(h.get_index(32), Some(31));
        assert_eq!(h.index_value(31), 32);
    }

    #[test]
    fn test_get_index_1() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 100,
            precision: 1,
        }).unwrap();

        assert_eq!(h.get_index(1), Some(0));
        assert_eq!(h.get_index(2), Some(1));
        assert_eq!(h.get_index(15), Some(14));

        // powers of two are 10 apart from value = 16 and up
        assert_eq!(h.get_index(16), Some(15));
        assert_eq!(h.get_index(32), Some(25));
        assert_eq!(h.get_index(64), Some(35));

        // this tests that rounding up within inner bucket works
        assert_eq!(h.get_index(16), Some(15));
        assert_eq!(h.get_index(17), Some(15));
        assert_eq!(h.get_index(18), Some(16));
        assert_eq!(h.get_index(19), Some(16));

        // these values prove roll-up into next outer bucket works
        assert_eq!(h.get_index(62), Some(34));
        assert_eq!(h.get_index(63), Some(34));
        assert_eq!(h.get_index(64), Some(35));

        assert_eq!(h.get_index(65), Some(35));
    }

    #[test]
    fn test_get_index_2() {
        // extensive test from precomputed table
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 100,
            precision: 1,
        }).unwrap();

        let v = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 21, 23, 24, 26, 28, 29,
            31, 32, 36, 39, 42, 45, 48, 52, 55, 58, 61, 64, 71, 77, 84, 90, 96, 103, 109, 116, 122
        ];

        for index in 0..45 {
            let got = h.get_index(v[index]).unwrap();
            assert!(got == index, "Value: {} Got: {} Want: {}", v[index], got, index);
        }

        for index in 0..45 {
            let got = h.index_value(index);
            assert!(got == v[index], "Index: {} Got: {} Want: {}", index, got, v[index]);
        }
    }

    #[test]
    fn test_get_index_3() {
        // extensive test from precomputed table
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 250,
            precision: 1,
        }).unwrap();

        let v = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 21, 23, 24, 26, 28, 29,
            31, 32, 36, 39, 42, 45, 48, 52, 55, 58, 61, 64, 71, 77, 84, 90, 96, 103, 109, 116, 122,
            128, 141, 154, 167, 180, 192, 205, 218, 231, 244
        ];

        for index in 0..55 {
            let got = h.get_index(v[index]).unwrap();
            assert!(got == index, "Value: {} Got: {} Want: {}", v[index], got, index);
        }

        for index in 0..55 {
            let got = h.index_value(index);
            assert!(got == v[index], "Index: {} Got: {} Want: {}", index, got, v[index]);
        }
    }

    #[test]
    fn test_index_value_0() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 100,
            precision: 1,
        }).unwrap();

        assert_eq!(h.index_value(0), 1);
        assert_eq!(h.index_value(1), 2);
        assert_eq!(h.index_value(14), 15);

        assert_eq!(h.index_value(15), 16);
        assert_eq!(h.index_value(25), 32);
        assert_eq!(h.index_value(35), 64);
    }

    #[test]
    fn test_iterator() {
        let mut h = Histogram::new(HistogramConfig {
            max_memory: 0,
            max_value: 100,
            precision: 1,
        }).unwrap();

        loop {
            match h.next() {
                Some(bucket) => {
                    match h.get_index(bucket.value) {
                        Some(_) => {
                            h.increment(bucket.value);
                        },
                        None => {},
                    }
                },
                None => { break }
            }
        }
    }
}
