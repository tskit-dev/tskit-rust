## Miscellaneous operations

### Writing to a file

```rust, noplayground, ignore
treeseq.dump("file.trees", tskit::TableOutputOptions::default()).unwrap();
```

### Loading from a file

```rust, noplayground, ignore
let treeseq = tskit::TreeSequence::load("file.trees").unwrap();
```

### Get a deep copy of the tables

Get a *copy* of the table collection in the tree sequence:

```rust, noplayground, ignore
let tables = treeseq.dump_tables.unwrap();
```

This function can error because the `tskit-c` functions to copy ,may return an error code.

This function is not necessary to access the tables.
See below.

### Read-only table access

A `TreeSequence` has access to the tables.
For example:

```rust, noplayground, ignore
for _edge in treeseq.edges_iter() {
}
```


