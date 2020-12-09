# rust bindings for tskit

This crate provides [rust](https://www.rust-lang.org/) bindings to [tskit](https://github.com/tskit-dev/tskit).

Currently, we only provide low-level bindings to the C API of both `tskit` and `kastore`.
We use [bindgen](https://docs.rs/bindgen) to automatically generate the bindings.

The overview is:

1. `tskit` and `kastore` are submodules.
2. These two tools are compiled into the `rust` package.
3. Then `bindgen` generates the bindings.
4. Finally, the entire rust package is generated.

The result is a `rust` library with all of these two C libraries statically compiled in.
Further, `rust` types and functions exist in the module name `tskit_rust::bindings`, allowing `unsafe` access to the low-level API.

In the future, we hope to develop a more "rusty" front-end, hiding the `unsafe` bits from client code.

Help wanted!

## Quick start guide

```sh
git clone https://github.com/molpopgen/tskit_rust
cd tskit_rust
git submodule update --init --recursive
cargo build
cargo test
```

Then, to look at the docs:

```
cargo doc --open
```
