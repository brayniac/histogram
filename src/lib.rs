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
//! * provide functionality to merge histograms
//! * provide an iterator
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
}

#[allow(dead_code)]
pub struct HistogramProperties {
    buckets_inner: u32,
    buckets_outer: u32,
    buckets_total: u32,
    memory_used: u32,
    direct_index_max: u64,
    direct_index_power: u32,
}

pub struct Histogram {
    config: HistogramConfig,
    data: HistogramData,
    properties: HistogramProperties,
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

        let buckets_inner = radix.pow(config.precision);

        // we can directly index any value less than the number of inner buckets
        let direct_index_power = 31 - buckets_inner.leading_zeros();
        let direct_index_max = 2_u64.pow(direct_index_power);

        let shift = config.max_value.leading_zeros() + direct_index_power;

        let mut buckets_outer = 64;

        if shift <= 64 {
            buckets_outer = 64 - config.max_value.leading_zeros() - direct_index_power;
        }

        let buckets_total = buckets_inner * buckets_outer + direct_index_max as u32;

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
            },
            properties: HistogramProperties {
                buckets_inner: buckets_inner,
                buckets_outer: buckets_outer,
                buckets_total: buckets_total,
                memory_used: memory_used,
                direct_index_max: direct_index_max,
                direct_index_power: direct_index_power,
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

            if value < self.properties.direct_index_max {
                return Some((value - 1) as usize);
            }

            let outer = 63 - value.leading_zeros();

            let inner =
                self.properties.buckets_inner as f64 *
                (value as f64 - 2.0_f64.powi((outer) as i32)) / 2.0_f64.powi((outer) as i32);

            let outer = outer as u64;
            let inner = inner.ceil() as u64;

            let index = (outer - self.properties.direct_index_power as u64) *
                            self.properties.buckets_inner as u64 + inner +
                        self.properties.direct_index_max - 1;
            return Some(index as usize);
        }
        result
    }

    // calculate the nominal value of the given index
    fn index_value(&mut self, index: usize) -> u64 {

        if index < self.properties.direct_index_max as usize {
            return (index + 1) as u64;
        }

        let index = (index as u32 - self.properties.direct_index_max as u32 +
                     self.properties.direct_index_power * self.properties.buckets_inner as u32 +
                     1) as usize;

        let power = (index as f64 / self.properties.buckets_inner as f64).floor() as u32;

        let remain = index - (power * self.properties.buckets_inner) as usize;

        let mut value = 2.0_f64.powi((power as u32) as i32);
        value += remain as f64 * (value as f64 / self.properties.buckets_inner as f64);

        value.floor() as u64
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
            max_value: 1000000,
            precision: 2,
        }).unwrap();

        assert_eq!(h.get_index(1), Some(0));
        assert_eq!(h.index_value(0), 1);

        assert_eq!(h.get_index(2), Some(1));
        assert_eq!(h.index_value(1), 2);

        assert_eq!(h.get_index(128), Some(163));
        assert_eq!(h.index_value(163), 128);

        assert_eq!(h.get_index(256), Some(263));
        assert_eq!(h.get_index(255), Some(263));
        assert_eq!(h.get_index(257), Some(264));
        assert_eq!(h.get_index(258), Some(264));
    }
}
