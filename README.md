# histogram - histogram storage and percentile stats

histogram is a stats library for rust which provides histogram
storage with percentile stats. Maintains precision guarentees
throughout the range of stored values.

[![Build Status](https://travis-ci.org/brayniac/histogram.svg?branch=master)](https://travis-ci.org/brayniac/histogram)
[![crates.io](http://meritbadge.herokuapp.com/histogram)](https://crates.io/crates/mio)

## Usage

To use `histogram`, first add this to your `Cargo.toml`:

```toml
[dependencies]
histogram = "*"
```

Then, add this to your crate root:

```rust
extern crate histogram;
```

## Features

* Values are stored with precision guarentees
* Pre-allocated on initialization
* Retrieve percentile stats
