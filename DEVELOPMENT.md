# Developer's guide

## Targeted rust version

All development targets `stable`.

## rust tool chain

Install `rust` as described [here](https://www.rust-lang.org/tools/install).

Then, install [rustfmt](https://github.com/rust-lang/rustfmt) and [clippy](https://github.com/rust-lang/rust-clippy), which are required to make sure that changes can pass CI tests (see below):

```sh
rustup component add rustfmt clippy
```

### Other useful tools

[tarpaulin](https://docs.rs/crate/cargo-tarpaulin/) for calculating test coverage:

```sh
cargo install cargo-tarpaulin
```


## Continuous integration (CI)

GitHub actions/work flows handle all CI:

* `rustfmt` checks code formatting
* `clippy` checks for code "lint".
* The library is built, and tests are run, against both rust `stable` and `beta`.

### More about clippy

`clippy` is a very opinionated code "linter".
Any changes must pass all `clippy` checks.
Some of these checks may seem stylistic, meaning that you are being asked to replace working code with a different implementation.
In these cases `clippy` is suggesting a more idiomatic approach to do what you are asking.
We accept `clippy`'s recommendations for two reasons.
First, it is best to respect language idioms  whenever possible ("don't force rust to act like C or C++").
Second, these recommendations have been useful in learning more about rust.

## Building the library

```sh
cargo build
```

### Building examples

```sh
cargo build --examples
```

### Release mode builds

Add `--release` to any of the above.
This flags adds optimizations and removes debugging symbols.

## Building the documentation

```sh
cargo doc
```

Then, point your browser at `target/doc/tskit/index.html`.

## Running tests

To run tests and doc tests:

```sh
cargo test
```

To test examples:

```sh
cargo test --examples
```

### Test coverage

Using `tarpaulin`:

```sh
cargo tarpaulin --exclude-files '*.c' --exclude-files '*.h' -o html
```

We exclude `*.c` and `*.h` because it is `tskit`'s job to manage the coverage of those files.

Some notes on what `tarpaulin` does:

* The coverage includes the test code itself, which is a bit annoying.
  In the future, we may move all tests to a separate directory and exclude them from the calculation.

## Running optimized examples

The default build is `debug`, which makes the examples slow.
To run `release` builds of examples:

```sh
cargo build --release --examples
```

The binaries will be in `target/release/examples/`.

## Tips and tricks

### Cleaning up the various builds

```sh
cargo clean
```

### Cleaning out the dependency database

```sh
rm -f Cargo.lock
```
