## Iterating over edge differences

The API provides an iterator over edge differences.
Each step of the iterator advances to the next tree in the tree sequence.
For each tree, a standard `Iterator` over removals and insertions is available:

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:iterate_edge_differences}}
```

Edge differences are the basis of efficient algorithms based on the incremental updating of summaries of trees.
The following example co-iterates over edge differences and trees.
The edge differences are used to calculate the parent array for each tree:

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:iterate_edge_differences_update_parents}}
```


