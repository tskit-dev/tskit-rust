## Accessing table columns

We can access table rows using either the relevant newtype or `i32` (which is identical to the `tskit-c` typedef `tsk_id_t`).
The following snippet adds and edge and then validates the data for that row of the table:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_columns}}
```

The return type of the getters is the [`Option`](https://doc.rust-lang.org/std/option/enum.Option.html) enum. The `None` variant is returned when row indexes are out of range:

```rust, noplaygound, ignore
{{#include ../../tests/book_table_collection.rs:get_edge_table_columns_out_of_range}}
```
