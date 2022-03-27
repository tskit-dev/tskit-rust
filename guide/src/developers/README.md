# Developer's guide

## Installing rust

See [here](https://www.rust-lang.org/tools/install) and follow the appropriate instructions for your system.

## Running tests

To run the tests on default features:

```sh
cargo test
```

To test all features and tests found in `examples/`:

```sh
cargo test --all-targets --all-features
```

## Running clippy
