# Tables and tree sequences

## Creating a table collection (without metadata)

We will populate a table collection corresponding to the following trees. Tree 1 is not fully-coalesced,
having two roots defining two sub-trees:

```ignore
 0
+++
| |  1
| | +++
2 3 4 5
```

Tree 2 is fully coalesced:

```ignore
    0
  +-+-+
  1   |
+-+-+ |
2 4 5 3
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:init_table}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:add_first_node}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:add_second_node}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:add_sample_nodes}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:add_edges}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:sort_tables}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:index_tables}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:check_integrity_lite}}
```

```rust, noplayground, ignore
{{#include ../../examples/tree_traversals.rs:check_integrity_full}}
```

