## Working with trees

Iterating over a tree sequence returns instances of `tskit::Tree`.
This type is immutable.
No access is provided to the underlying pointers.

The API provides a set of iterators over the data in a tree.

For example, to collect the siblings of each node into a vector:

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:iterate_node_siblings}}
```

We may do the same calculation using API elements giving `&[NodeId]`
(slices of node ids).
This method more closely matches the `tskit-c` API.

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:iterate_node_siblings_via_arrays}}
```

This approach is more complex:

* Slice element access is via `usize` (`size_t` in C/C++).
* Row ids are `i32` (`tsk_id_t` in `tskit-c`) behind the newtypes.
* `tskit` implements `TryFrom` to help you out, forcing
  you to reckon with the fact that conversion from `i32`
  to `usize` is fallible.

These conversion semantics require us to manually handle all possible error
paths at each step.

We can have an intermediate level of complexity using getters from the tree arrays:

```rust, noplaygound, ignore
{{#include ../../tests/book_trees.rs:iterate_node_siblings_via_array_getters}}
```
