## Initialization from a table collection

The steps are:

* Add rows to a table
* Sort the table
* Index the table
* Create the tree sequence

For brevity, we skip careful pattern matching of return values
and instead just unwrap them.

### Adding rows

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:build_tables}}
```

### Sorting

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:sort_tables}}
```

See the [API docs](https://docs.rs) for the details of sorting.
The behavior of this function can be confusing at first.
Only tables with strict sorting requirements are affected.

### Indexing

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:index_tables}}
```

### Create the tree sequence

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:create_tree_sequence}}
```

Notes:

* We could have skipped `tables.build_index()` and passed `TreeSquenceFlags::BUILD_INDEXES` instead of the default flags.
* Creating a tree sequence from a table collection takes ownership of the table data and consumes the table collection variable.
  Any further attempts to manipulate the table collection variable will result in a compiler error.
