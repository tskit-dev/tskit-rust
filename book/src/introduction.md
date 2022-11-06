# Introduction <img align="right" width="73" height="45" src="https://raw.githubusercontent.com/tskit-dev/administrative/main/logos/svg/tskit-rust/Tskit_rust_logo.eps.svg">

## Required background.

* We assume familiarity with `tskit`. See [tskit.dev](https://tskit.dev).
* Comfort with rust is required.
  * [The book](https://doc.rust-lang.org/book/) is a good place to start.
  * [The Rustonomicon](https://doc.rust-lang.org/nomicon/) is for those who wish to dig deeper into `unsafe` rust and FFI.
    The 'nomicon is helpful for understanding how things like the `tskit` rust API can be built in the first place.

## Conventions used in this document

Here, `tskit` means the rust API.
`tskit-c` and `tskit-python` refer to the C and Python APIs, respectively.

The phrase "data model" refers to [this](https://tskit.dev/tskit/docs/stable/data-model.html). This document will make little sense without an understanding of the data model.

## Relation to the C API

Where necessary, we will note differences from the behavior of `tskit-c`.

Much of the rust API works by calling the C API.
We do not change the semantics of the C API.
However, we do make stricter statements about the ownership relationships between types.
For example, the C API can result in the following situation:

* A heap-allocated table collection is used to record data about the ancestry of a sample.
* That table collection is used to initialize a tree sequence.
  The tree sequence is told to take ownership of the tables.

This is a case where the C API requires that you respectfully no longer work with the heap-allocated table collection. To do so is undefined behavior.

The rust API forbids such situations.
The creation of a tree sequence from tables consumes the tables via a move operation.
Thus, any further actions on the tables is a compiler error.

This example is the kinds of differences between `tskit` and `tskit-c`.
Undefined behavior is (close to) impossible with `tskit`.


## Quick overview

### Do you need `tskit-rust`?

The use-cases for `tskit-rust` are the same as for `tskit-c`:

1. Developing a new performance-oriented application.
2. The input/output of this application will be a `.trees` file.

### What does `tskit-rust` add?

Briefly, you get the performance of `C` and the strong safety guarantees of `rust`.

### What is `tskit-rust` missing?

The crate does not cover the entire `C` API.
However, client code can make direct calls to that API via the module `tskit::bindings`.

### Adding `tskit` as a dependency to a rust project

In your `Cargo.toml`:

```{toml}
[dependencies]
tskit = "~X.Y.Z"
```

The latest version to fill in `X.Y.Z` can be found [here](https://crates.io/crates/tskit).
See [here](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) for how to specify version numbers.

#### Feature flags.

`tskit` defines several [cargo features](https://doc.rust-lang.org/cargo/reference/features.html).
These are defined in the [API docs](https://docs.rs/tskit/latest/tskit/#optional-features).

