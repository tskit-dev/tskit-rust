# rust bindings for tskit <img align="right" width="145" height="90" src="https://raw.githubusercontent.com/tskit-dev/administrative/main/logos/svg/tskit-rust/Tskit_rust_logo.eps.svg">

![CI tests](https://github.com/molpopgen/tskit_rust/workflows/CI/badge.svg)

This crate provides [rust](https://www.rust-lang.org/) bindings to [tskit](https://github.com/tskit-dev/tskit).

This package provides the following:

1. Low-level bindings to the C API of both `tskit` and `kastore`.
   We use [bindgen](https://docs.rs/bindgen) to automatically generate the bindings.
2. Support for table collections, tree sequences, and tree iteration.
3. An error handling system that maps `tskit` error
   codes to `rust` errors while preserving error messages.

The overview is:

1. `tskit` and `kastore` C code are include in `subprojects/`
2. These two tools are compiled into the `rust` package.
3. Then `bindgen` generates the bindings.
4. Finally, the entire rust package is generated.

The result is a `rust` library with all of these two C libraries statically compiled in.
Further, `rust` types and functions exist in the module name `tskit::bindings`, allowing `unsafe` access to the low-level API.

Help wanted!

## Quick start guide

### Cloning the repository and running the test suite

```sh
git clone https://github.com/tskit-dev/tskit-rust
cd tskit-rust
cargo test --all-features
```

### Viewing the documentation

```
cargo doc --all-features --open
```

### Calculating code coverage

First, install `tarpaulin`:

```sh
cargo install cargo-tarpaulin
```

Then, we use all tests, doc tests, and example programs to calculate code coverage for all available features:

```sh
cargo tarpaulin --all-features --doc --tests --examples --exclude-files '*.c' --exclude-files '*.h' --ignore-tests  -o html
```

Then, point your favorite browser to `tarpaulin-report.html`.

The last few flags exclude the C code and any `rust` code that is test-only from being part of the denominator of the coverage calculation.
The goal here is not to have high *test* coverage of the C API, as it is up to the [upstream project](https://github.com/tskit-dev/tskit) to provide that.

**Note:** `tarpaulin` can be fickle, and changing the order of some of those flags can cause the coverage run to fail.

## Change log

See [here](https://github.com/tskit-dev/tskit-rust/blob/main/CHANGELOG.md).

