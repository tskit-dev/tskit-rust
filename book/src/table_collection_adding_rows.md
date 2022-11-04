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
