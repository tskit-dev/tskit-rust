## Accessing table rows

The rows of a table contain two types of data:

* Numerical data consisting of a single value.
* Ragged data. Examples include metadata for all tables,
  ancestral states for sites, derived states for mutations,
  parent and location information for individuals, etc..

`tskit` provides two ways to access row data.
The first is by a "view", which contains non-owning references
to the ragged column data.
The second is by row objects containing *copies* of the ragged column data.

The former will be more efficient when the ragged columns are populated.
The latter will be more convenient to work with because the API is a standard
rust iterator.

By holding references, row views have the usual implications for borrowing.
The row objects, however, own their data and are thus independent of their parent
objects.

### Row views

To generate a row view using a row id:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_row_by_id}}
```

To iterate over all views we use *lending* iterators:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_rows_by_lending_iterator}}
```

#### Lending iterators

The lending iterators are implemented using the [`streaming_iterator`](https://docs.rs/streaming-iterator/latest/streaming_iterator/) crate.
(The community now prefers the term "lending" over "streaming" for this concept.)
The `tskit` prelude includes the trait declarations that allow the code shown above to compile.

rust 1.65.0 stabilized Generic Associated Types, or GATs.
GATs allows lending iterators to be implemented directly without the workarounds used in the `streaming_iterator` crate.
We have decided not to implement our own lending iterator using GATs.
Rather, we will see what the community settles on and will decide in the future whether or not to adopt it.

### Row objects

We may access entire table rows by a row id:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_row_by_id}}
```

The row types are rust structures. The table data are *copied* into these structures, making them relatively more expensive to work with. We are looking into removing the copies and returning slices, but doing so currently impacts the design of table row iterators such as:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_rows_by_iterator}}
```

These iterators are rust iterator types--they `impl Iterator<Item=X>`, where `X` is a table row type.
