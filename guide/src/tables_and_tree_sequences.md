# Tables and tree sequences

We will populate a table collection corresponding to the following trees. Tree 1 is not fully-coalesced,
having two roots defining two sub-trees:

```
 0
+++
| |  1
| | +++
2 3 4 5
```

Tree 2 is fully coalesced:

```
    0
  +-+-+
  1   |
+-+-+ |
2 4 5 3
```

```rust
{{#rustdoc_include ../../examples/tree_traversals.rs:test}}
```

```rust
{{#rustdoc_include tables_and_tree_sequences.rs:addfirstnode}}
```

```rust
{{#rustdoc_include tables_and_tree_sequences.rs:addremainingnodes}}
```

```rust
{{#rustdoc_include tables_and_tree_sequences.rs:addedges}}
```

```rust
{{#include ../../examples/tree_traversals.rs}}
```


