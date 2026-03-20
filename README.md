# histogram

A collection of histogram data structures for Rust, providing standard, atomic,
and sparse variants. Like HDRHistogram, values are stored in quantized buckets,
but the bucket construction and indexing algorithm are modified for fast
increments and lookups.

## Getting Started

```
cargo add histogram
```

## Usage

```rust
use histogram::Histogram;

// Create a histogram with grouping power 7 and max value power 64.
let mut histogram = Histogram::new(7, 64).unwrap();

// Record some values.
for i in 1..=100 {
    histogram.increment(i).unwrap();
}

// Query percentiles using the 0.0..=1.0 scale.
let median = histogram.percentile(0.5).unwrap().unwrap();
let p99 = histogram.percentile(0.99).unwrap().unwrap();
// percentile() returns Result<Option<Bucket>, Error>
// outer unwrap: percentile value is valid
// inner unwrap: histogram is non-empty

println!("median: {}", median.end());
println!("p99: {}", p99.end());
```

## Histogram Types

- **Histogram** -- Standard histogram with plain 64-bit counters. Best for
  single-threaded use.
- **AtomicHistogram** -- Uses atomic 64-bit counters, allowing concurrent
  recording from multiple threads. Take a snapshot via `load()` or `drain()`
  to query percentiles.
- **SparseHistogram** -- Columnar representation that only stores non-zero
  buckets. Ideal for serialization and storage when most buckets are empty.

## Features

- `serde` -- Enables `Serialize` and `Deserialize` for histogram types.
- `schemars` -- Enables JSON Schema generation (implies `serde`).

## Documentation

- [API Documentation](https://docs.rs/histogram)
- [Crates.io](https://crates.io/crates/histogram)
- [Repository](https://github.com/iopsystems/histogram)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
