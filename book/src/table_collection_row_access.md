## Accessing table rows

We may also access entire table rows by a row id:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_row_by_id}}
```

The row types are rust structures. The table data are *copied* into these structures, making them relatively more expensive to work with. We are looking into removing the copies and returning slices, but doing so currently impacts the design of table row iterators such as:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_rows_by_iterator}}
```

These iterators are rust iterator types--they `impl Iterator<Item=X>`, where `X` is a table row type.
