# Defining metadata types in rust

A key feature of the API is that metadata is specified on a per-table basis.
In other words, a type to be used as node metadata implements the `tskit::metadata::NodeMetadata` trait.

Using the `tskit` cargo feature `derive`, we can use procedural macros to define metadata types.
Here, we define a metadata type for a mutation table:

```rust, noplayground, ignore
{{#include ../../tests/book_metadata.rs:metadata_derive}}
```

We require that you also manually specify the `serde` derive macros because the metadata API
itself does not depend on `serde`.
Rather, it expects raw bytes and `serde` happens to be a good way to get them from your data types.

The derive macro also enforces some helpful behavior at compile time.
You will get a compile-time error if you try to derive two different metadata types for the same rust type.
The error is due to conflicting implementations for a [supertrait](https://doc.rust-lang.org/rust-by-example/trait/supertraits.html).
