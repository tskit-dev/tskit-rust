## Adding rows to tables

* For each table type (node, edge., etc.), we have a function to add a row.
* We can only add rows to tables in mutable `TableCollection` instances.

For example, to add a node:


```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:add_node_without_metadata}}
```

We see from the `if let` pattern that functions adding rows return
[`Result`](https://doc.rust-lang.org/std/result/enum.Result.html).
In general, errors only occur when the C back-end fails to allocate memory 
to expand the table columns.
If we add a row with invalid data, no error is returned!
To catch such errors, we must explicitly check table integrity (see [below](table_collection_validation.md#checking-table-integrity)).

Again, we can take advantage of being able to pass in any type that is `Into<_>` the required newtype:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:add_node_without_metadata_using_into}}
```

See the [API docs](https://docs.rs/tskit) for more details and examples.

### Adding nodes using default values

This section is more advanced and may be skipped during a first read.

For some tables it may be common to input the same values over and over for some fields when adding rows.
Let's take a look at how to use default values when adding rows to a node table.

Default instances of `NodeDefaults` contain default values for the `flags`, `individual`, and `population` fields:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_defaults}}
```

Add a node with these values and a given birth time:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:add_node_defaults}}
```

We can use struct update syntax to create a new node marked as a sample while re-using our other defaults:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:add_node_defaults_sample}}
```

See the [`NodeDefaults`](https://docs.rs/tskit/latest/tskit/struct.NodeDefaults.html) section of the API reference for more.

#### Metadata

[Metadata](metadata.md#Metadata) can complicate the picture a bit:

* Metadata types are defined by the client and are thus a generic in the `tskit` API.
* We do not want to impose too many trait bounds on the client-defined types.
* Metadata is optional on a per-row basis for any given table.

[`NodeDefaultsWithMetadata`](https://docs.rs/tskit/latest/tskit/struct.NodeDefaultsWithMetadata.html) handles the case where rows may or may not have metadata.
The metadata type is generic with trait bound [`tskit::NodeMetadata`](https://docs.rs/tskit/latest/tskit/metadata/trait.NodeMetadata.html).
Because metadata are optional per-row, any metadata defaults are stored as an [`Option`](https://doc.rust-lang.org/std/option/).

For the following examples, this will be our metadata type:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_metadata}}
```

##### Case 1: no default metadata

A common use case is that the metadata differs for every row.
For this case, it makes sense for the default value to be the `None` variant of the `Option`. 

This case is straightforward:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_defaults_with_metadata}}
```

##### Case 2: default metadata

TL;DR:

* If table row defaults *include* metadata, you can run into use-after-move issues.
  Fortunately, the compiler will catch this as an error.
* The solution is for your metadata type to implement `Clone`.

Consider the following case:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_defaults_with_some_metadata_default}}
```

Imagine that the first row we add uses different metadata but all the other default values:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_defaults_with_some_metadata_default_add_first_row}}
```

Nothing interesting has happened.
However, let's take a look at what we need to do if our next row uses a non-default `population` field and the default metadata:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_defaults_with_some_metadata_default_add_second_row}}
```

Note the call to `..defaults.clone()`.
(For that call to compile, `NodeMetadata` must implement `Clone`!.) 
Without that, our `defaults` instance would have *moved*, leading to a move-after-use compiler error when we add a third row:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:node_defaults_with_some_metadata_default_add_third_row}}
```

