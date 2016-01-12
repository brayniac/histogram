# histogram - histogram storage and percentile stats

histogram is a stats library for rust which provides histogram
storage with percentile stats. Maintains precision guarentees
throughout the range of stored values.

[![Build Status](https://travis-ci.org/brayniac/histogram.svg?branch=master)](https://travis-ci.org/brayniac/histogram)
[![crates.io](http://meritbadge.herokuapp.com/histogram)](https://crates.io/crates/histogram)
[![License](http://img.shields.io/:license-mit-blue.svg)](http://doge.mit-license.org)

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

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
