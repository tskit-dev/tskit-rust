# rust bindings for tskit

![CI tests](https://github.com/molpopgen/tskit_rust/workflows/CI/badge.svg)

This crate provides [rust](https://www.rust-lang.org/) bindings to [tskit](https://github.com/tskit-dev/tskit).

This package provides the following:

1. Low-level bindings to the C API of both `tskit` and `kastore`.
   We use [bindgen](https://docs.rs/bindgen) to automatically generate the bindings.
2. The struct `tskit_rust::TableCollection`, which wraps
   `tsk_table_collection_t`. The wrapper is currently
   incomplete, supporting only node, edge, site,
   mutation, and population tables.  The remaining
   tables will happen "soon".
3. An error handling system that maps `tskit` error
   codes to `rust` errors while preserving error messages.

The overview is:

1. `tskit` and `kastore` source from `tskit 0.3.4` are include in `subprojects/`
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
